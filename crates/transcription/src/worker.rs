use std::sync::Arc;

use bson::oid::ObjectId;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info, warn};

use crate::asr::{AsrBackend, AsrRequest};
use crate::config::TranscriptionConfig;
use crate::pipeline::rtp_parser::RtpPacket;
use crate::pipeline::{OpusDecoder, Resampler};
use crate::TranscriptEvent;

#[cfg(feature = "vad")]
use crate::vad::{SileroVad, VadEvent};

/// A speech segment produced by the ingestion loop for the ASR loop.
struct SpeechSegment {
    audio: Vec<f32>,
    start_time: f64,
    end_time: f64,
}

/// Per-producer async pipeline task.
///
/// Receives RTP packets from mediasoup DirectTransport, processes them through:
/// RTP parse → Opus decode → Resample (48kHz→16kHz) → VAD → [channel] → ASR → TranscriptEvent
///
/// The ingestion loop and ASR loop run as separate tasks so that RTP processing
/// is never blocked by ASR inference.
#[allow(dead_code)]
pub struct TranscriptionWorker {
    user_id: ObjectId,
    conference_id: ObjectId,
    speaker_name: String,
    asr: Arc<dyn AsrBackend>,
    config: TranscriptionConfig,
    rtp_rx: mpsc::Receiver<Vec<u8>>,
    transcript_tx: broadcast::Sender<TranscriptEvent>,
}

impl TranscriptionWorker {
    pub fn new(
        user_id: ObjectId,
        conference_id: ObjectId,
        speaker_name: String,
        asr: Arc<dyn AsrBackend>,
        config: TranscriptionConfig,
        rtp_rx: mpsc::Receiver<Vec<u8>>,
        transcript_tx: broadcast::Sender<TranscriptEvent>,
    ) -> Self {
        Self {
            user_id,
            conference_id,
            speaker_name,
            asr,
            config,
            rtp_rx,
            transcript_tx,
        }
    }

    /// Runs the worker pipeline until the RTP channel is closed.
    ///
    /// Spawns an ingestion task (RTP → VAD) that feeds speech segments through a channel
    /// to the ASR loop, so RTP processing is never blocked by ASR inference.
    pub async fn run(self) {
        info!(
            user_id = %self.user_id,
            conference_id = %self.conference_id,
            speaker = %self.speaker_name,
            backend = %self.asr.name(),
            "Transcription worker started"
        );

        let (segment_tx, segment_rx) = mpsc::channel::<SpeechSegment>(16);

        let config_clone = self.config.clone();
        let rtp_rx = self.rtp_rx;
        let ingestion = tokio::spawn(Self::ingestion_loop(rtp_rx, config_clone, segment_tx));

        Self::asr_loop(
            segment_rx,
            self.asr,
            self.config,
            self.user_id,
            self.conference_id,
            self.speaker_name,
            self.transcript_tx,
        )
        .await;

        ingestion.abort();

        debug!("Transcription worker stopped");
    }

    /// Ingestion loop: RTP parse → Opus decode → Resample → VAD → SpeechSegment.
    ///
    /// Runs independently so that incoming RTP packets are always processed even
    /// while the ASR loop is busy with inference.
    async fn ingestion_loop(
        mut rtp_rx: mpsc::Receiver<Vec<u8>>,
        config: TranscriptionConfig,
        segment_tx: mpsc::Sender<SpeechSegment>,
    ) {
        let mut opus_decoder = match OpusDecoder::new() {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to create Opus decoder: {}", e);
                return;
            }
        };

        let mut resampler = match Resampler::new(960) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to create resampler: {}", e);
                return;
            }
        };

        #[cfg(feature = "vad")]
        let mut vad = {
            let vad_path = config
                .vad_model_path
                .as_deref()
                .unwrap_or("models/silero_vad.onnx");
            match SileroVad::new(vad_path, &config) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to create VAD: {}", e);
                    return;
                }
            }
        };

        let mut last_seq: Option<u16> = None;
        let mut rtp_count: u64 = 0;
        #[cfg(feature = "vad")]
        let start_time = std::time::Instant::now();

        while let Some(rtp_data) = rtp_rx.recv().await {
            rtp_count += 1;
            if rtp_count == 1 || rtp_count % 500 == 0 {
                info!(rtp_count, bytes = rtp_data.len(), "RTP packets received");
            }

            // 1. Parse RTP
            let rtp_packet = match RtpPacket::parse(&rtp_data) {
                Some(p) => p,
                None => {
                    warn!("Invalid RTP packet, skipping");
                    continue;
                }
            };

            let payload = rtp_packet.payload(&rtp_data);
            if payload.is_empty() {
                continue;
            }

            // Check for packet loss
            if let Some(prev) = last_seq {
                let expected = prev.wrapping_add(1);
                if rtp_packet.header.sequence_number != expected {
                    let gap = rtp_packet
                        .header
                        .sequence_number
                        .wrapping_sub(prev)
                        .wrapping_sub(1);
                    debug!(gap, "RTP packet loss detected, running PLC");
                    for _ in 0..gap.min(3) {
                        if let Ok(pcm) = opus_decoder.decode_plc() {
                            if let Ok(resampled) = resampler.process(&pcm) {
                                #[cfg(feature = "vad")]
                                {
                                    let _ = vad.process(&resampled);
                                }
                                #[cfg(not(feature = "vad"))]
                                let _ = resampled;
                            }
                        }
                    }
                }
            }
            last_seq = Some(rtp_packet.header.sequence_number);

            // 2. Decode Opus → 48kHz mono PCM
            let pcm_48k = match opus_decoder.decode_to_mono(payload) {
                Ok(p) => p,
                Err(e) => {
                    warn!("Opus decode error: {}", e);
                    continue;
                }
            };

            // 3. Resample 48kHz → 16kHz
            let pcm_16k = match resampler.process(&pcm_48k) {
                Ok(p) => p,
                Err(e) => {
                    warn!("Resample error: {}", e);
                    continue;
                }
            };

            if pcm_16k.is_empty() {
                continue;
            }

            // 4. Feed to VAD
            #[cfg(feature = "vad")]
            {
                let events = vad.process(&pcm_16k);
                for event in events {
                    match event {
                        VadEvent::SpeechEnd {
                            audio,
                            duration_secs,
                        } => {
                            info!(
                                duration_secs,
                                samples = audio.len(),
                                "Speech segment ended, sending to ASR"
                            );

                            let elapsed = start_time.elapsed().as_secs_f64();
                            let segment = SpeechSegment {
                                audio,
                                start_time: elapsed - duration_secs,
                                end_time: elapsed,
                            };

                            if segment_tx.send(segment).await.is_err() {
                                debug!("ASR loop closed, stopping ingestion");
                                return;
                            }
                        }
                    }
                }
            }

            #[cfg(not(feature = "vad"))]
            {
                let _ = pcm_16k;
            }
        }

        debug!("RTP channel closed, ingestion loop exiting");
    }

    /// ASR loop: receives speech segments, runs transcription, emits TranscriptEvents.
    async fn asr_loop(
        mut segment_rx: mpsc::Receiver<SpeechSegment>,
        asr: Arc<dyn AsrBackend>,
        config: TranscriptionConfig,
        user_id: ObjectId,
        conference_id: ObjectId,
        speaker_name: String,
        transcript_tx: broadcast::Sender<TranscriptEvent>,
    ) {
        while let Some(segment) = segment_rx.recv().await {
            let request = AsrRequest {
                audio_pcm_16k_mono: segment.audio,
                language_hint: config.language.clone(),
                sample_rate: 16000,
            };

            let start = std::time::Instant::now();
            match asr.transcribe(request).await {
                Ok(result) => {
                    let inference_duration_ms = start.elapsed().as_millis() as u64;
                    let text = result.text.trim().to_string();
                    if text.is_empty() {
                        debug!("ASR returned empty text, skipping");
                        continue;
                    }

                    let event = TranscriptEvent {
                        conference_id,
                        user_id,
                        speaker_name: speaker_name.clone(),
                        text,
                        language: result.language,
                        confidence: result.confidence,
                        start_time: segment.start_time,
                        end_time: segment.end_time,
                        inference_duration_ms,
                    };

                    if let Err(e) = transcript_tx.send(event) {
                        debug!("No transcript subscribers: {}", e);
                    }
                }
                Err(e) => {
                    warn!("ASR transcription error: {}", e);
                }
            }
        }
    }
}
