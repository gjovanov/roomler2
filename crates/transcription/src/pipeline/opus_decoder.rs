use std::convert::TryFrom;

use audiopus::coder::Decoder;
use audiopus::packet::Packet;
use audiopus::{Channels, MutSignals, SampleRate};

/// Wraps libopus decoder. Decodes Opus packets into f32 PCM at 48kHz.
pub struct OpusDecoder {
    decoder: Decoder,
    /// Interleaved stereo output buffer (max 120ms at 48kHz stereo = 11520 samples).
    decode_buf: Vec<f32>,
}

/// Maximum Opus frame: 120ms at 48kHz = 5760 samples/channel, stereo = 11520.
const MAX_FRAME_SIZE: usize = 5760 * 2;

impl OpusDecoder {
    pub fn new() -> anyhow::Result<Self> {
        let decoder = Decoder::new(SampleRate::Hz48000, Channels::Stereo)
            .map_err(|e| anyhow::anyhow!("Failed to create Opus decoder: {:?}", e))?;
        Ok(Self {
            decoder,
            decode_buf: vec![0.0f32; MAX_FRAME_SIZE],
        })
    }

    /// Decodes an Opus packet into mono f32 PCM at 48kHz.
    ///
    /// Returns the decoded mono samples. Stereo input is down-mixed to mono.
    pub fn decode_to_mono(&mut self, opus_data: &[u8]) -> anyhow::Result<Vec<f32>> {
        let output = MutSignals::try_from(&mut self.decode_buf[..])
            .map_err(|e| anyhow::anyhow!("MutSignals error: {:?}", e))?;

        let packet = Packet::try_from(opus_data)
            .map_err(|e| anyhow::anyhow!("Packet error: {:?}", e))?;

        let samples_per_channel = self
            .decoder
            .decode_float(Some(packet), output, false)
            .map_err(|e| anyhow::anyhow!("Opus decode error: {:?}", e))?;

        // Down-mix interleaved stereo [L,R,L,R,...] to mono
        let mut mono = Vec::with_capacity(samples_per_channel);
        for i in 0..samples_per_channel {
            let left = self.decode_buf[i * 2];
            let right = self.decode_buf[i * 2 + 1];
            mono.push((left + right) * 0.5);
        }

        Ok(mono)
    }

    /// Generates a PLC (Packet Loss Concealment) frame.
    /// Call this when an RTP packet is lost (gap in sequence numbers).
    pub fn decode_plc(&mut self) -> anyhow::Result<Vec<f32>> {
        let output = MutSignals::try_from(&mut self.decode_buf[..])
            .map_err(|e| anyhow::anyhow!("MutSignals error: {:?}", e))?;

        // Opus PLC: pass None as input
        let samples_per_channel = self
            .decoder
            .decode_float(None, output, false)
            .map_err(|e| anyhow::anyhow!("Opus PLC error: {:?}", e))?;

        let mut mono = Vec::with_capacity(samples_per_channel);
        for i in 0..samples_per_channel {
            let left = self.decode_buf[i * 2];
            let right = self.decode_buf[i * 2 + 1];
            mono.push((left + right) * 0.5);
        }

        Ok(mono)
    }
}
