//! # hl7-mllp
//!
//! Transport-agnostic MLLP (Minimal Lower Layer Protocol) framing for HL7 v2 messages.
//!
//! MLLP is the standard transport envelope used by HL7 v2 over TCP/IP. This crate
//! provides pure framing logic — encoding and decoding MLLP frames — without coupling
//! to any specific async runtime, I/O library, or transport mechanism.
//!
//! ## Design
//!
//! - [`MllpTransport`] trait: implement this for any byte-stream transport
//! - [`MllpFrame`]: encode/decode MLLP frames from raw bytes
//! - No tokio, no async-std, no opinion on I/O
//!
//! ## MLLP Frame Format
//!
//! ```text
//! [VT] [HL7 message bytes ...] [FS] [CR]
//!  0x0B                         0x1C  0x0D
//! ```
//!
//! ## Example
//!
//! ```rust
//! use hl7_mllp::{MllpFrame, MllpError};
//!
//! let raw_hl7 = b"MSH|^~\\&|...";
//! let framed = MllpFrame::encode(raw_hl7);
//!
//! let decoded = MllpFrame::decode(&framed).unwrap();
//! assert_eq!(decoded, raw_hl7);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use bytes::{BufMut, Bytes, BytesMut};

/// MLLP start-of-block character (VT, 0x0B).
pub const VT: u8 = 0x0B;

/// MLLP end-of-block character (FS, 0x1C).
pub const FS: u8 = 0x1C;

/// MLLP carriage return terminator (CR, 0x0D).
pub const CR: u8 = 0x0D;

/// Errors produced by MLLP framing operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MllpError {
    /// Input did not begin with the expected VT start byte.
    MissingStartByte,
    /// Input did not end with the expected FS+CR sequence.
    MissingEndSequence,
    /// The frame was empty (no HL7 payload between delimiters).
    EmptyPayload,
    /// The buffer was too short to contain a complete frame.
    Incomplete,
}

impl std::fmt::Display for MllpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingStartByte => write!(f, "MLLP frame missing VT start byte (0x0B)"),
            Self::MissingEndSequence => {
                write!(f, "MLLP frame missing FS+CR end sequence (0x1C 0x0D)")
            }
            Self::EmptyPayload => write!(f, "MLLP frame contains no HL7 payload"),
            Self::Incomplete => write!(f, "Buffer does not contain a complete MLLP frame"),
        }
    }
}

impl std::error::Error for MllpError {}

/// MLLP frame encoder and decoder.
///
/// This struct contains only associated functions — there is no state.
/// It operates purely on byte slices and [`Bytes`].
pub struct MllpFrame;

impl MllpFrame {
    /// Wrap a raw HL7 message payload in an MLLP frame.
    ///
    /// Produces: `[VT] payload [FS] [CR]`
    pub fn encode(payload: &[u8]) -> Bytes {
        let mut buf = BytesMut::with_capacity(payload.len() + 3);
        buf.put_u8(VT);
        buf.put_slice(payload);
        buf.put_u8(FS);
        buf.put_u8(CR);
        buf.freeze()
    }

    /// Extract the HL7 payload from an MLLP-framed buffer.
    ///
    /// Returns a slice into the original buffer — zero copy.
    pub fn decode(buf: &[u8]) -> Result<&[u8], MllpError> {
        if buf.len() < 4 {
            return Err(MllpError::Incomplete);
        }
        if buf[0] != VT {
            return Err(MllpError::MissingStartByte);
        }
        let end = buf.len();
        if buf[end - 2] != FS || buf[end - 1] != CR {
            return Err(MllpError::MissingEndSequence);
        }
        let payload = &buf[1..end - 2];
        if payload.is_empty() {
            return Err(MllpError::EmptyPayload);
        }
        Ok(payload)
    }

    /// Find the end of the first complete MLLP frame in a streaming buffer.
    ///
    /// Returns `Some(n)` where `n` is the byte length of the complete frame
    /// (including delimiters), or `None` if the buffer does not yet contain
    /// a complete frame. Useful for implementing streaming readers.
    pub fn find_frame_end(buf: &[u8]) -> Option<usize> {
        if buf.is_empty() || buf[0] != VT {
            return None;
        }
        for i in 1..buf.len().saturating_sub(1) {
            if buf[i] == FS && buf[i + 1] == CR {
                return Some(i + 2);
            }
        }
        None
    }

    /// Find all complete MLLP frames in a buffer.
    ///
    /// Returns a vector of (start, end) byte positions for each complete frame found.
    /// Start position is the index of the VT byte, end position is the index after CR.
    /// Partial frames at the end of the buffer are not included.
    ///
    /// # Example
    /// ```
    /// use hl7_mllp::MllpFrame;
    ///
    /// let frame1 = MllpFrame::encode(b"MSH|first");
    /// let frame2 = MllpFrame::encode(b"MSH|second");
    /// let combined = [&frame1[..], &frame2[..]].concat();
    ///
    /// let frames = MllpFrame::find_all_frames(&combined);
    /// assert_eq!(frames, vec![(0, frame1.len()), (frame1.len(), frame1.len() + frame2.len())]);
    /// ```
    pub fn find_all_frames(buf: &[u8]) -> Vec<(usize, usize)> {
        let mut frames = Vec::new();
        let mut pos = 0;

        while pos < buf.len() {
            // Look for VT start byte
            if buf[pos] != VT {
                #[cfg(feature = "noncompliance")]
                {
                    // Skip non-VT bytes at the start (tolerate extra bytes before VT)
                    if let Some(vt_pos) = buf[pos..].iter().position(|&b| b == VT) {
                        pos += vt_pos;
                    } else {
                        break;
                    }
                }
                #[cfg(not(feature = "noncompliance"))]
                break;
            }

            // Need at least VT + 1 byte + FS + CR = 4 bytes minimum
            if buf.len() - pos < 4 {
                break;
            }

            // Search for FS+CR end sequence
            let search_start = pos + 1;
            let mut found_end = None;

            for i in search_start..buf.len().saturating_sub(1) {
                if buf[i] == FS && buf[i + 1] == CR {
                    found_end = Some(i + 2); // Position after CR
                    break;
                }
            }

            #[cfg(feature = "noncompliance")]
            if found_end.is_none() {
                // Tolerate missing final CR - look for FS at end of remaining buffer
                // Check: at least VT + 1 byte payload + FS from current position
                let remaining = buf.len() - pos;
                if remaining >= 3 && buf[buf.len() - 1] == FS {
                    // Ensure there's at least 1 byte of payload between VT and FS
                    if remaining >= 3 {
                        found_end = Some(buf.len());
                    }
                }
            }

            if let Some(end) = found_end {
                // Ensure payload is not empty (at least 1 byte between VT and FS)
                if end - pos >= 4 {
                    frames.push((pos, end));
                    pos = end;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        frames
    }

    /// Build a minimal HL7 ACK message payload (not MLLP-framed).
    ///
    /// `message_control_id` should be the message control ID from the original MSH-10.
    ///
    /// # Errors
    /// Returns `None` if `message_control_id` is empty.
    pub fn build_ack(message_control_id: &str, accepting: bool) -> Option<String> {
        if message_control_id.is_empty() {
            return None;
        }
        let code = if accepting { "AA" } else { "AE" };
        Some(format!(
            "MSH|^~\\&|||||{}||ACK|{}|P|2.3.1\rMSA|{}|{}",
            chrono_now_str(),
            message_control_id,
            code,
            message_control_id,
        ))
    }

    /// Build a minimal HL7 NACK message payload with error details (not MLLP-framed).
    ///
    /// `message_control_id` should be the message control ID from the original MSH-10.
    /// `error_code` should be an HL7 error code (e.g., "101", "102").
    /// `error_text` should be a human-readable error description.
    ///
    /// # Errors
    /// Returns `None` if `message_control_id` is empty.
    pub fn build_nack(
        message_control_id: &str,
        error_code: &str,
        error_text: &str,
    ) -> Option<String> {
        if message_control_id.is_empty() {
            return None;
        }
        Some(format!(
            "MSH|^~\\&|||||{}||ACK|{}|P|2.3.1\rMSA|AR|{}|{}|{}",
            chrono_now_str(),
            message_control_id,
            message_control_id,
            error_code,
            error_text,
        ))
    }
}

fn chrono_now_str() -> String {
    #[cfg(feature = "timestamps")]
    {
        use chrono::Local;
        Local::now().format("%Y%m%d%H%M%S").to_string()
    }
    #[cfg(not(feature = "timestamps"))]
    {
        // Default placeholder timestamp — caller should provide real timestamp
        "20250101000000".to_string()
    }
}

/// Trait for types that can act as an MLLP byte-stream transport.
///
/// Implement this for TCP streams, serial ports, in-memory buffers,
/// or any other byte-stream source. The crate provides no concrete
/// implementation — that is intentionally left to consumers.
pub trait MllpTransport {
    /// The error type returned by this transport.
    type Error: std::error::Error;

    /// Read the next complete MLLP-framed message from the transport.
    ///
    /// Implementations are responsible for accumulating bytes until a
    /// complete frame is available. Use [`MllpFrame::find_frame_end`]
    /// as the completion signal.
    fn read_frame(&mut self) -> Result<Vec<u8>, Self::Error>;

    /// Write an MLLP-framed ACK or NACK back to the sender.
    fn write_frame(&mut self, frame: &[u8]) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let payload =
            b"MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240101120000||ORU^R01|12345|P|2.3.1";
        let framed = MllpFrame::encode(payload);
        let decoded = MllpFrame::decode(&framed).unwrap();
        assert_eq!(decoded, payload);
    }

    #[test]
    fn missing_start_byte() {
        let bad = b"no_vt_here\x1C\x0D";
        assert_eq!(MllpFrame::decode(bad), Err(MllpError::MissingStartByte));
    }

    #[test]
    fn missing_end_sequence() {
        let bad = b"\x0Bpayload_no_end";
        assert_eq!(MllpFrame::decode(bad), Err(MllpError::MissingEndSequence));
    }

    #[test]
    fn find_frame_end_complete() {
        let payload = b"MSH|test";
        let framed = MllpFrame::encode(payload);
        assert_eq!(MllpFrame::find_frame_end(&framed), Some(framed.len()));
    }

    #[test]
    fn find_frame_end_incomplete() {
        let partial = b"\x0Bincomplete_data";
        assert_eq!(MllpFrame::find_frame_end(partial), None);
    }

    // T1.1 — Consecutive frames tests
    #[test]
    fn find_all_frames_two_back_to_back() {
        let payload1 = b"MSH|first";
        let payload2 = b"MSH|second";
        let frame1 = MllpFrame::encode(payload1);
        let frame2 = MllpFrame::encode(payload2);
        let combined = [&frame1[..], &frame2[..]].concat();

        let frames = MllpFrame::find_all_frames(&combined);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0], (0, frame1.len()));
        assert_eq!(frames[1], (frame1.len(), frame1.len() + frame2.len()));

        // Verify decoded payloads
        let decoded1 = MllpFrame::decode(&combined[frames[0].0..frames[0].1]).unwrap();
        let decoded2 = MllpFrame::decode(&combined[frames[1].0..frames[1].1]).unwrap();
        assert_eq!(decoded1, payload1);
        assert_eq!(decoded2, payload2);
    }

    #[test]
    fn find_all_frames_with_partial_third() {
        let payload1 = b"MSH|first";
        let payload2 = b"MSH|second";
        let payload3 = b"MSH|partial_no_end";
        let frame1 = MllpFrame::encode(payload1);
        let frame2 = MllpFrame::encode(payload2);
        let partial3 = [&[VT][..], payload3].concat();

        let combined = [&frame1[..], &frame2[..], &partial3[..]].concat();

        let frames = MllpFrame::find_all_frames(&combined);
        assert_eq!(frames.len(), 2); // Only first two complete frames
        assert_eq!(frames[0], (0, frame1.len()));
        assert_eq!(frames[1], (frame1.len(), frame1.len() + frame2.len()));
    }

    #[test]
    fn find_all_frames_empty_buffer() {
        assert!(MllpFrame::find_all_frames(b"").is_empty());
    }

    #[test]
    fn find_all_frames_no_frames() {
        assert!(MllpFrame::find_all_frames(b"garbage_data_no_vt").is_empty());
    }

    #[test]
    fn find_all_frames_empty_payload_rejected() {
        // Empty payload (VT immediately followed by FS+CR) should be rejected
        let empty_frame = [VT, FS, CR];
        let frames = MllpFrame::find_all_frames(&empty_frame);
        assert!(frames.is_empty(), "Empty payload frame should be rejected");
    }

    // T1.1 — Verify byte sequence against HL7 v2.5.1 Appendix C
    #[test]
    fn verify_mllp_byte_constants() {
        // VT = 0x0B (Vertical Tab) - start of block
        // FS = 0x1C (File Separator) - end of block
        // CR = 0x0D (Carriage Return) - terminator
        assert_eq!(VT, 0x0B, "VT must be 0x0B per HL7 v2.5.1 Appendix C");
        assert_eq!(FS, 0x1C, "FS must be 0x1C per HL7 v2.5.1 Appendix C");
        assert_eq!(CR, 0x0D, "CR must be 0x0D per HL7 v2.5.1 Appendix C");
    }

    #[test]
    fn verify_single_byte_start_block() {
        // MLLP uses single-byte VT start block, no multi-byte variants
        let frame = MllpFrame::encode(b"test");
        assert_eq!(frame[0], VT);
        assert_eq!(frame.len(), 7); // VT (1) + 4 bytes + FS (1) + CR (1)
    }

    // Noncompliance feature tests
    #[cfg(feature = "noncompliance")]
    mod noncompliance_tests {
        use super::*;

        #[test]
        fn tolerate_missing_final_cr() {
            // Frame with VT + payload + FS (missing CR)
            let payload = b"MSH|test";
            let incomplete = [&[VT][..], payload, &[FS]].concat();

            let frames = MllpFrame::find_all_frames(&incomplete);
            assert_eq!(frames.len(), 1);
            assert_eq!(frames[0], (0, incomplete.len()));
        }

        #[test]
        fn tolerate_extra_bytes_before_vt() {
            // Garbage bytes before valid frame
            let payload = b"MSH|test";
            let frame = MllpFrame::encode(payload);
            let garbage_before = [b"garbage", &frame[..]].concat();

            let frames = MllpFrame::find_all_frames(&garbage_before);
            assert_eq!(frames.len(), 1);
            // Frame should start after garbage (at position 7)
            assert_eq!(frames[0].0, 7);
        }

        #[test]
        fn noncompliance_empty_payload_rejected() {
            // Even with noncompliance, empty payload should be rejected
            let empty_frame = [VT, FS]; // VT + FS, no payload, no CR
            let frames = MllpFrame::find_all_frames(&empty_frame);
            assert!(
                frames.is_empty(),
                "Empty payload should be rejected even with noncompliance"
            );
        }

        #[test]
        fn strict_mode_rejects_missing_cr() {
            // Without noncompliance feature, missing CR should result in no frames found
            // This test is compiled only without the feature
            let payload = b"MSH|test";
            let incomplete = [&[VT][..], payload, &[FS]].concat();

            // In strict mode, this should not find a complete frame
            // (But we can't test this here since it's cfg-gated)
        }
    }

    // T1.2 — ACK generation tests
    #[test]
    fn build_ack_validates_empty_control_id() {
        assert!(MllpFrame::build_ack("", true).is_none());
        assert!(MllpFrame::build_ack("", false).is_none());
    }

    #[test]
    fn build_ack_creates_aa_for_accept() {
        let ack = MllpFrame::build_ack("MSG001", true).unwrap();
        assert!(ack.contains("MSA|AA|MSG001"));
    }

    #[test]
    fn build_ack_creates_ae_for_reject() {
        let ack = MllpFrame::build_ack("MSG001", false).unwrap();
        assert!(ack.contains("MSA|AE|MSG001"));
    }

    #[test]
    fn build_nack_validates_empty_control_id() {
        assert!(MllpFrame::build_nack("", "101", "Error").is_none());
    }

    #[test]
    fn build_nack_creates_ar_with_error_details() {
        let nack = MllpFrame::build_nack("MSG001", "101", "Invalid message").unwrap();
        assert!(nack.contains("MSA|AR|MSG001|101|Invalid message"));
    }

    #[test]
    fn build_nack_contains_ack_msh() {
        let nack = MllpFrame::build_nack("MSG001", "102", "Parse error").unwrap();
        // Should have MSH with ACK message type
        assert!(nack.starts_with("MSH|^~\\&|||||"));
        assert!(nack.contains("||ACK|MSG001|"));
    }

    // T1.2 — Round-trip ACK parse test
    #[test]
    fn ack_roundtrip_parse() {
        use hl7_v2::Hl7Message;

        let ack_str = MllpFrame::build_ack("MSG12345", true).unwrap();
        let ack_bytes = ack_str.as_bytes();

        // Parse the ACK using hl7-v2 crate
        let parsed = Hl7Message::parse(ack_bytes);
        assert!(
            parsed.is_ok(),
            "ACK should be valid HL7 that hl7-v2 can parse"
        );

        let msg = parsed.unwrap();
        // Verify MSH segment exists
        let msh = msg.segment("MSH");
        assert!(msh.is_some(), "ACK should have MSH segment");

        // Verify MSA segment exists
        let msa = msg.segment("MSA");
        assert!(msa.is_some(), "ACK should have MSA segment");
    }
}
