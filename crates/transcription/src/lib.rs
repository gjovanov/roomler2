pub mod asr;
pub mod config;
pub mod engine;
pub mod pipeline;
#[cfg(feature = "vad")]
pub mod vad;
pub mod worker;

pub use asr::{AsrBackend, AsrRequest, TranscriptionResult};
pub use config::TranscriptionConfig;
pub use engine::TranscriptionEngine;

use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// A transcription event emitted when an utterance is transcribed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEvent {
    pub conference_id: ObjectId,
    pub user_id: ObjectId,
    pub speaker_name: String,
    pub text: String,
    pub language: Option<String>,
    pub confidence: Option<f64>,
    /// Seconds since conference transcription started.
    pub start_time: f64,
    /// Seconds since conference transcription started.
    pub end_time: f64,
    /// How long ASR inference took in milliseconds.
    pub inference_duration_ms: u64,
}
