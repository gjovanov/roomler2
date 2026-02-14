use async_trait::async_trait;

use super::{AsrBackend, AsrRequest, TranscriptionResult};

/// NVIDIA NIM remote ASR backend via gRPC.
///
/// Connects to a NIM container running Canary-1B or Canary-Qwen-2.5B models
/// via the Riva ASR gRPC API.
pub struct RemoteNimBackend {
    endpoint: String,
}

impl RemoteNimBackend {
    pub fn new(endpoint: &str) -> anyhow::Result<Self> {
        Ok(Self {
            endpoint: endpoint.to_string(),
        })
    }
}

#[async_trait]
impl AsrBackend for RemoteNimBackend {
    async fn transcribe(&self, _request: AsrRequest) -> anyhow::Result<TranscriptionResult> {
        // TODO: Implement gRPC client using tonic + prost with Riva ASR proto definitions.
        // For now, return a placeholder indicating the backend is not yet implemented.
        Err(anyhow::anyhow!(
            "RemoteNimBackend not yet implemented (endpoint: {})",
            self.endpoint
        ))
    }

    fn name(&self) -> &str {
        "remote_nim"
    }

    fn supports_language(&self, lang: &str) -> bool {
        matches!(lang, "en" | "de" | "fr" | "es")
    }
}
