use mediasoup::prelude::*;
use serde::{Deserialize, Serialize};

/// Client -> Server signaling messages (sent over WebSocket).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientSignal {
    /// Client requests to join a media room
    #[serde(rename = "media:join")]
    MediaJoin { conference_id: String },

    /// Client provides DTLS parameters to connect a transport
    #[serde(rename = "media:connect_transport")]
    ConnectTransport {
        conference_id: String,
        transport_id: String,
        dtls_parameters: DtlsParameters,
    },

    /// Client requests to produce media (audio/video/screen)
    #[serde(rename = "media:produce")]
    Produce {
        conference_id: String,
        kind: MediaKind,
        rtp_parameters: RtpParameters,
    },

    /// Client requests to consume a remote producer
    #[serde(rename = "media:consume")]
    Consume {
        conference_id: String,
        producer_id: String,
        rtp_capabilities: RtpCapabilities,
    },

    /// Client closes a specific producer
    #[serde(rename = "media:producer_close")]
    ProducerClose {
        conference_id: String,
        producer_id: String,
    },

    /// Client leaves the media room
    #[serde(rename = "media:leave")]
    MediaLeave { conference_id: String },
}

/// Server -> Client signaling messages (sent over WebSocket).
/// These are not deserialized by the server; they're serialized and sent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerSignal {
    /// Router RTP capabilities for Device loading
    #[serde(rename = "media:router_capabilities")]
    RouterCapabilities {
        rtp_capabilities: serde_json::Value,
    },

    /// Send + recv transport pair created
    #[serde(rename = "media:transport_created")]
    TransportCreated {
        send_transport: super::room_manager::TransportOptions,
        recv_transport: super::room_manager::TransportOptions,
    },

    /// Producer creation result
    #[serde(rename = "media:produce_result")]
    ProduceResult { id: String },

    /// Consumer created for a remote producer
    #[serde(rename = "media:consumer_created")]
    ConsumerCreated {
        id: String,
        producer_id: String,
        kind: String,
        rtp_parameters: serde_json::Value,
    },

    /// A new producer appeared in the room (notify to trigger consume)
    #[serde(rename = "media:new_producer")]
    NewProducer {
        producer_id: String,
        user_id: String,
        kind: String,
    },

    /// A peer left the media room
    #[serde(rename = "media:peer_left")]
    PeerLeft {
        user_id: String,
        conference_id: String,
    },

    /// A producer was closed
    #[serde(rename = "media:producer_closed")]
    ProducerClosed {
        producer_id: String,
        user_id: String,
    },

    /// Error response
    #[serde(rename = "media:error")]
    Error { message: String },
}
