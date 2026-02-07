use serde::{Deserialize, Serialize};

/// WebRTC signaling message types.
/// Full implementation in Phase 5 with mediasoup integration.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum SignalingMessage {
    /// Client requests to join a media room
    MediaJoin {
        conference_id: String,
    },
    /// Server sends router RTP capabilities
    RouterCapabilities {
        rtp_capabilities: serde_json::Value,
    },
    /// Client sends transport connection parameters
    ConnectTransport {
        transport_id: String,
        dtls_parameters: serde_json::Value,
    },
    /// Client requests to produce media
    Produce {
        transport_id: String,
        kind: String,
        rtp_parameters: serde_json::Value,
    },
    /// Server notifies of a new producer
    NewProducer {
        producer_id: String,
        user_id: String,
        kind: String,
    },
    /// Client requests to consume a producer
    Consume {
        producer_id: String,
    },
    /// Server sends consumer parameters
    ConsumerCreated {
        consumer_id: String,
        producer_id: String,
        kind: String,
        rtp_parameters: serde_json::Value,
    },
    /// Client leaves the media room
    MediaLeave {
        conference_id: String,
    },
}
