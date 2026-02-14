use std::collections::HashMap;
use std::sync::Arc;

use bson::oid::ObjectId;
use dashmap::DashMap;
use tokio::sync::{broadcast, mpsc, Mutex};
use tracing::{debug, info, warn};

use crate::asr::AsrBackend;
use crate::config::TranscriptionConfig;
use crate::worker::TranscriptionWorker;
use crate::TranscriptEvent;

/// Manages per-producer transcription pipelines with multi-backend support.
///
/// The engine is created once at startup and shared via `Arc`. It supports
/// multiple named ASR backends and per-conference model selection.
pub struct TranscriptionEngine {
    /// Named ASR backends (e.g. "whisper" -> LocalWhisperBackend, "canary" -> LocalOnnxBackend).
    backends: HashMap<String, Arc<dyn AsrBackend>>,
    /// Default backend name.
    default_backend: String,
    config: TranscriptionConfig,
    /// Active worker tasks, keyed by producer_id string.
    workers: DashMap<String, WorkerHandle>,
    /// Broadcast channel for transcript events.
    transcript_tx: broadcast::Sender<TranscriptEvent>,
    /// Per-conference model selection: conference_id -> backend name.
    conference_models: Mutex<HashMap<ObjectId, String>>,
}

struct WorkerHandle {
    abort_handle: tokio::task::AbortHandle,
}

impl TranscriptionEngine {
    /// Creates a new multi-backend transcription engine.
    ///
    /// Returns `(engine, transcript_receiver)`.
    pub fn new(
        backends: HashMap<String, Arc<dyn AsrBackend>>,
        default_backend: String,
        config: TranscriptionConfig,
    ) -> (Arc<Self>, broadcast::Receiver<TranscriptEvent>) {
        let (transcript_tx, transcript_rx) = broadcast::channel(256);

        let backend_names: Vec<&String> = backends.keys().collect();
        info!(
            ?backend_names,
            default = %default_backend,
            "Transcription engine created"
        );

        let engine = Arc::new(Self {
            backends,
            default_backend,
            config,
            workers: DashMap::new(),
            transcript_tx,
            conference_models: Mutex::new(HashMap::new()),
        });

        (engine, transcript_rx)
    }

    /// Returns a new broadcast receiver for transcript events.
    pub fn subscribe(&self) -> broadcast::Receiver<TranscriptEvent> {
        self.transcript_tx.subscribe()
    }

    /// Enables transcription for a conference with a specific model.
    pub async fn enable_conference(&self, conference_id: ObjectId, model_name: String) {
        let mut models = self.conference_models.lock().await;
        models.insert(conference_id, model_name.clone());
        info!(%conference_id, model = %model_name, "Transcription enabled for conference");
    }

    /// Disables transcription for a conference and stops all its workers.
    pub async fn disable_conference(&self, conference_id: ObjectId) {
        {
            let mut models = self.conference_models.lock().await;
            models.remove(&conference_id);
        }

        // Stop all workers for this conference
        let to_remove: Vec<String> = self
            .workers
            .iter()
            .filter(|entry| entry.key().starts_with(&conference_id.to_hex()))
            .map(|entry| entry.key().clone())
            .collect();

        for key in to_remove {
            self.stop_pipeline(&key);
        }

        info!(%conference_id, "Transcription disabled for conference");
    }

    /// Checks if transcription is enabled for a conference.
    pub async fn is_enabled(&self, conference_id: &ObjectId) -> bool {
        let models = self.conference_models.lock().await;
        models.contains_key(conference_id)
    }

    /// Gets the ASR backend for a conference, falling back to default.
    fn get_backend(&self, conference_id: &ObjectId, models: &HashMap<ObjectId, String>) -> Option<Arc<dyn AsrBackend>> {
        let model_name = models
            .get(conference_id)
            .unwrap_or(&self.default_backend);

        if let Some(backend) = self.backends.get(model_name) {
            return Some(backend.clone());
        }

        // Fallback: try default backend
        if let Some(backend) = self.backends.get(&self.default_backend) {
            warn!(
                requested = %model_name,
                fallback = %self.default_backend,
                "Requested backend not found, using default"
            );
            return Some(backend.clone());
        }

        // Fallback: try any available backend
        if let Some((name, backend)) = self.backends.iter().next() {
            warn!(fallback = %name, "Default backend not found, using first available");
            return Some(backend.clone());
        }

        None
    }

    /// Starts a transcription pipeline for an audio producer.
    ///
    /// If a pipeline already exists for this producer, it is stopped first
    /// (handles model switching).
    pub fn start_pipeline(
        self: &Arc<Self>,
        conference_id: ObjectId,
        producer_id: String,
        user_id: ObjectId,
        speaker_name: String,
        rtp_rx: mpsc::Receiver<Vec<u8>>,
    ) {
        let key = format!("{}:{}", conference_id.to_hex(), producer_id);

        // Stop any existing pipeline for this producer (e.g. model switch)
        if self.workers.contains_key(&key) {
            info!(%key, "Replacing existing pipeline (model switch)");
            self.stop_pipeline(&key);
        }

        // Resolve backend synchronously using try_lock
        let asr = {
            let models = match self.conference_models.try_lock() {
                Ok(guard) => guard,
                Err(_) => {
                    // If we can't lock, use default backend directly
                    match self.backends.get(&self.default_backend)
                        .or_else(|| self.backends.values().next())
                        .cloned()
                    {
                        Some(backend) => {
                            self.spawn_worker(key, conference_id, producer_id, user_id, speaker_name, backend, rtp_rx);
                            return;
                        }
                        None => {
                            warn!("No ASR backends available");
                            return;
                        }
                    }
                }
            };
            match self.get_backend(&conference_id, &models) {
                Some(b) => b,
                None => {
                    warn!("No ASR backends available for conference {}", conference_id);
                    return;
                }
            }
        };

        self.spawn_worker(key, conference_id, producer_id, user_id, speaker_name, asr, rtp_rx);
    }

    fn spawn_worker(
        self: &Arc<Self>,
        key: String,
        conference_id: ObjectId,
        _producer_id: String,
        user_id: ObjectId,
        speaker_name: String,
        asr: Arc<dyn AsrBackend>,
        rtp_rx: mpsc::Receiver<Vec<u8>>,
    ) {
        debug!(%key, backend = %asr.name(), "Starting transcription pipeline");

        let worker = TranscriptionWorker::new(
            user_id,
            conference_id,
            speaker_name.clone(),
            asr,
            self.config.clone(),
            rtp_rx,
            self.transcript_tx.clone(),
        );

        // Spawn worker and auto-cleanup on completion
        let cleanup_key = key.clone();
        let engine = Arc::clone(self);
        let handle = tokio::spawn(async move {
            worker.run().await;
            // Remove from workers map when done (natural exit or RTP channel closed)
            engine.workers.remove(&cleanup_key);
            debug!(%cleanup_key, "Worker entry cleaned up");
        });

        self.workers.insert(
            key.clone(),
            WorkerHandle {
                abort_handle: handle.abort_handle(),
            },
        );

        debug!(%key, %speaker_name, "Transcription pipeline started");
    }

    /// Stops a transcription pipeline by its key (conference_id:producer_id).
    pub fn stop_pipeline(&self, key: &str) {
        if let Some((_, handle)) = self.workers.remove(key) {
            handle.abort_handle.abort();
            debug!(%key, "Transcription pipeline stopped");
        }
    }

    /// Stops the pipeline for a specific producer.
    pub fn stop_producer(&self, conference_id: &ObjectId, producer_id: &str) {
        let key = format!("{}:{}", conference_id.to_hex(), producer_id);
        self.stop_pipeline(&key);
    }

    /// Returns the number of active pipelines.
    pub fn active_pipeline_count(&self) -> usize {
        self.workers.len()
    }

    /// Returns the list of available backend names.
    pub fn available_backends(&self) -> Vec<String> {
        self.backends.keys().cloned().collect()
    }
}
