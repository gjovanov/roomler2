use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tracing::debug;

use super::canary::CanaryModel;
use super::{AsrBackend, AsrRequest, TranscriptionResult};

/// Local ONNX ASR backend using Canary-1B-v2.
///
/// Wraps the Canary encoder-decoder model behind an `Arc<Mutex>` because
/// ONNX `Session::run()` requires `&mut self` and `spawn_blocking` needs `'static`.
pub struct LocalOnnxBackend {
    model: Arc<Mutex<CanaryModel>>,
}

impl LocalOnnxBackend {
    pub fn new(model_dir: &str) -> anyhow::Result<Self> {
        let model = CanaryModel::from_pretrained(model_dir)?;
        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }
}

#[async_trait]
impl AsrBackend for LocalOnnxBackend {
    async fn transcribe(&self, request: AsrRequest) -> anyhow::Result<TranscriptionResult> {
        let audio = request.audio_pcm_16k_mono;
        let language_hint = request.language_hint;
        let samples = audio.len();
        let model = self.model.clone();

        let (text, detected_lang) = tokio::task::spawn_blocking(move || {
            let mut guard = model
                .lock()
                .map_err(|e| anyhow::anyhow!("Model lock poisoned: {}", e))?;
            guard.transcribe(&audio, language_hint.as_deref())
        })
        .await??;

        debug!(samples, text_len = text.len(), ?detected_lang, "Canary transcription complete");

        Ok(TranscriptionResult {
            text,
            language: detected_lang,
            confidence: None,
        })
    }

    fn name(&self) -> &str {
        "canary"
    }

    fn supports_language(&self, lang: &str) -> bool {
        matches!(lang, "en" | "de" | "fr" | "es")
    }
}
