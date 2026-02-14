/// Parsed RTP header fields.
#[derive(Debug, Clone)]
pub struct RtpHeader {
    pub version: u8,
    pub padding: bool,
    pub extension: bool,
    pub csrc_count: u8,
    pub marker: bool,
    pub payload_type: u8,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub ssrc: u32,
}

/// Parsed RTP packet: header + payload slice offset.
#[derive(Debug)]
pub struct RtpPacket {
    pub header: RtpHeader,
    /// Byte offset where the payload starts in the original packet.
    pub payload_offset: usize,
}

impl RtpPacket {
    /// Parses an RTP packet from raw bytes.
    ///
    /// Returns `None` if the data is too short or has an invalid version.
    pub fn parse(data: &[u8]) -> Option<Self> {
        // Minimum RTP header is 12 bytes
        if data.len() < 12 {
            return None;
        }

        let version = (data[0] >> 6) & 0x03;
        if version != 2 {
            return None;
        }

        let padding = (data[0] >> 5) & 0x01 != 0;
        let extension = (data[0] >> 4) & 0x01 != 0;
        let csrc_count = data[0] & 0x0F;
        let marker = (data[1] >> 7) & 0x01 != 0;
        let payload_type = data[1] & 0x7F;

        let sequence_number = u16::from_be_bytes([data[2], data[3]]);
        let timestamp = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let ssrc = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        // Payload starts after fixed header + CSRC list
        let mut offset = 12 + (csrc_count as usize) * 4;

        if offset > data.len() {
            return None;
        }

        // Skip extension header if present
        if extension {
            if offset + 4 > data.len() {
                return None;
            }
            // Extension header: 2 bytes profile + 2 bytes length (in 32-bit words)
            let ext_length = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;
            offset += 4 + ext_length * 4;

            if offset > data.len() {
                return None;
            }
        }

        Some(RtpPacket {
            header: RtpHeader {
                version,
                padding,
                extension,
                csrc_count,
                marker,
                payload_type,
                sequence_number,
                timestamp,
                ssrc,
            },
            payload_offset: offset,
        })
    }

    /// Extracts the payload bytes from the original packet data.
    pub fn payload<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        if self.payload_offset >= data.len() {
            return &[];
        }

        let end = if self.header.padding {
            // Last byte indicates padding length
            let pad_len = *data.last().unwrap_or(&0) as usize;
            if pad_len > 0 && data.len() >= pad_len {
                data.len() - pad_len
            } else {
                data.len()
            }
        } else {
            data.len()
        };

        &data[self.payload_offset..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rtp_packet(payload: &[u8]) -> Vec<u8> {
        let mut pkt = vec![
            0x80, // V=2, P=0, X=0, CC=0
            0x6F, // M=0, PT=111 (Opus)
            0x00, 0x01, // seq=1
            0x00, 0x00, 0x03, 0xE8, // timestamp=1000
            0x12, 0x34, 0x56, 0x78, // SSRC
        ];
        pkt.extend_from_slice(payload);
        pkt
    }

    #[test]
    fn parse_basic_rtp() {
        let data = make_rtp_packet(&[0xDE, 0xAD]);
        let pkt = RtpPacket::parse(&data).unwrap();
        assert_eq!(pkt.header.version, 2);
        assert_eq!(pkt.header.payload_type, 111);
        assert_eq!(pkt.header.sequence_number, 1);
        assert_eq!(pkt.header.timestamp, 1000);
        assert_eq!(pkt.header.ssrc, 0x12345678);
        assert_eq!(pkt.payload(&data), &[0xDE, 0xAD]);
    }

    #[test]
    fn reject_too_short() {
        assert!(RtpPacket::parse(&[0x80, 0x00]).is_none());
    }

    #[test]
    fn reject_wrong_version() {
        let mut data = make_rtp_packet(&[]);
        data[0] = 0x00; // version 0
        assert!(RtpPacket::parse(&data).is_none());
    }

    #[test]
    fn parse_with_csrc() {
        let pkt = vec![
            0x82, // V=2, P=0, X=0, CC=2
            0x6F, // PT=111
            0x00, 0x01, // seq
            0x00, 0x00, 0x00, 0x00, // timestamp
            0x00, 0x00, 0x00, 0x01, // SSRC
            // 2 CSRC entries (8 bytes)
            0x00, 0x00, 0x00, 0x02,
            0x00, 0x00, 0x00, 0x03,
            // Payload
            0xAA, 0xBB,
        ];
        let parsed = RtpPacket::parse(&pkt).unwrap();
        assert_eq!(parsed.header.csrc_count, 2);
        assert_eq!(parsed.payload_offset, 20);
        assert_eq!(parsed.payload(&pkt), &[0xAA, 0xBB]);
    }

    #[test]
    fn parse_with_extension() {
        let pkt = vec![
            0x90, // V=2, P=0, X=1, CC=0
            0x6F, // PT=111
            0x00, 0x01, // seq
            0x00, 0x00, 0x00, 0x00, // timestamp
            0x00, 0x00, 0x00, 0x01, // SSRC
            // Extension header
            0xBE, 0xDE, // profile-specific
            0x00, 0x01, // length = 1 (32-bit word)
            0x00, 0x00, 0x00, 0x00, // extension data
            // Payload
            0xCC,
        ];
        let parsed = RtpPacket::parse(&pkt).unwrap();
        assert!(parsed.header.extension);
        assert_eq!(parsed.payload(&pkt), &[0xCC]);
    }

    #[test]
    fn parse_with_padding() {
        let pkt = vec![
            0xA0, // V=2, P=1, X=0, CC=0
            0x6F,
            0x00, 0x01,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
            // Payload
            0xDD, 0xEE,
            // Padding (3 bytes, last byte = count)
            0x00, 0x00, 0x03,
        ];
        let parsed = RtpPacket::parse(&pkt).unwrap();
        assert!(parsed.header.padding);
        assert_eq!(parsed.payload(&pkt), &[0xDD, 0xEE]);
    }
}
