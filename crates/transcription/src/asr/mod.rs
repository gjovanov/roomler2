#[cfg(feature = "local-whisper")]
pub mod local_whisper;

#[cfg(feature = "local-onnx")]
pub mod canary;

#[cfg(feature = "local-onnx")]
pub mod local_onnx;

#[cfg(feature = "remote-nim")]
pub mod remote_nim;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Request to transcribe an audio segment.
pub struct AsrRequest {
    /// PCM audio at 16kHz mono, f32 normalized [-1.0, 1.0].
    pub audio_pcm_16k_mono: Vec<f32>,
    /// Optional language hint (ISO 639-1, e.g. "en", "de").
    pub language_hint: Option<String>,
    /// Sample rate (always 16000 for this pipeline).
    pub sample_rate: u32,
}

/// Result of an ASR transcription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Option<String>,
    pub confidence: Option<f64>,
}

/// Trait for pluggable ASR backends.
#[async_trait]
pub trait AsrBackend: Send + Sync + 'static {
    /// Transcribes a complete utterance (post-VAD).
    async fn transcribe(&self, request: AsrRequest) -> anyhow::Result<TranscriptionResult>;

    /// Human-readable backend name.
    fn name(&self) -> &str;

    /// Whether this backend supports a given language code.
    fn supports_language(&self, lang: &str) -> bool;
}
