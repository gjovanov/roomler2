use async_trait::async_trait;
use tracing::{debug, info};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use super::{AsrBackend, AsrRequest, TranscriptionResult};

/// Local Whisper ASR backend using whisper.cpp via whisper-rs.
pub struct LocalWhisperBackend {
    ctx: WhisperContext,
    default_language: Option<String>,
}

impl LocalWhisperBackend {
    /// Creates a new Whisper backend, loading the model from disk.
    ///
    /// `model_path` should point to a GGML Whisper model file (e.g. ggml-base.en.bin).
    pub fn new(model_path: &str, default_language: Option<String>) -> anyhow::Result<Self> {
        info!(model_path, "Loading Whisper model");
        let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
            .map_err(|e| anyhow::anyhow!("Failed to load Whisper model '{}': {}", model_path, e))?;
        info!("Whisper model loaded");
        Ok(Self {
            ctx,
            default_language,
        })
    }
}

#[async_trait]
impl AsrBackend for LocalWhisperBackend {
    async fn transcribe(&self, request: AsrRequest) -> anyhow::Result<TranscriptionResult> {
        let audio = request.audio_pcm_16k_mono;
        let lang = request
            .language_hint
            .or_else(|| self.default_language.clone());

        // whisper-rs is CPU-bound; run on blocking thread pool
        let ctx_ptr = &self.ctx as *const WhisperContext;
        // SAFETY: WhisperContext is Send+Sync, and we create a new state per call
        let ctx_ref = unsafe { &*ctx_ptr };

        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<TranscriptionResult> {
            let mut state = ctx_ref
                .create_state()
                .map_err(|e| anyhow::anyhow!("Failed to create Whisper state: {}", e))?;

            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

            if let Some(ref lang) = lang {
                params.set_language(Some(lang));
            }

            // Suppress non-speech output
            params.set_print_progress(false);
            params.set_print_special(false);
            params.set_print_realtime(false);
            params.set_print_timestamps(false);

            // Single-segment mode for short utterances
            params.set_single_segment(true);

            state
                .full(params, &audio)
                .map_err(|e| anyhow::anyhow!("Whisper transcription failed: {}", e))?;

            let n_segments = state.full_n_segments();

            let mut text = String::new();
            for i in 0..n_segments {
                if let Some(segment) = state.get_segment(i) {
                    if let Ok(seg_text) = segment.to_str() {
                        text.push_str(seg_text);
                    }
                }
            }

            let text = text.trim().to_string();
            debug!(text_len = text.len(), "Whisper transcription complete");

            Ok(TranscriptionResult {
                text,
                language: lang,
                confidence: None,
            })
        })
        .await
        .map_err(|e| anyhow::anyhow!("Whisper task join error: {}", e))??;

        Ok(result)
    }

    fn name(&self) -> &str {
        "local_whisper"
    }

    fn supports_language(&self, _lang: &str) -> bool {
        true // Whisper supports 99+ languages
    }
}
