# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-12

### Added

- **MLLP framing**: `MllpFrame::encode()` and `MllpFrame::decode()` with VT/FS/CR delimiters per HL7 v2.5.1 Appendix C
- **Streaming support**: `MllpFramer` struct for incremental frame accumulation from network streams
- **ACK/NACK generation**: `MllpFrame::build_ack()` and `MllpFrame::build_nack()` with proper HL7 message structure
- **Optional chrono integration**: `timestamps` feature for automatic HL7 DTM format timestamps
- **Noncompliance feature**: Tolerant parsing mode for handling non-compliant senders
- **MllpTransport trait**: For implementing custom transports (TCP, serial, etc.)
- **Error handling**: `MllpError` enum with `From<MllpError> for std::io::Error` conversion
- **Comprehensive documentation**: Module-level docs, examples, and TCP listener sample
- **39 unit tests** covering all functionality including Unicode payloads and edge cases

### API

```rust
// Framing
pub fn MllpFrame::encode(payload: &[u8]) -> Bytes;
pub fn MllpFrame::decode(buf: &[u8]) -> Result<&[u8], MllpError>;
pub fn MllpFrame::find_frame_end(buf: &[u8]) -> Option<usize>;
pub fn MllpFrame::find_all_frames(buf: &[u8]) -> Vec<(usize, usize)>;

// Streaming
pub struct MllpFramer;
pub fn MllpFramer::new() -> Self;
pub fn MllpFramer::push(&mut self, bytes: &[u8]);
pub fn MllpFramer::next_frame(&mut self) -> Option<Vec<u8>>;

// ACK/NACK
pub fn MllpFrame::build_ack(message_control_id: &str, accepting: bool) -> Option<String>;
pub fn MllpFrame::build_nack(message_control_id: &str, error_code: &str, error_text: &str) -> Option<String>;

// Transport trait
pub trait MllpTransport {
    type Error: std::error::Error;
    fn read_frame(&mut self) -> Result<Vec<u8>, Self::Error>;
    fn write_frame(&mut self, frame: &[u8]) -> Result<(), Self::Error>;
}
```

## [0.0.1] - 2025-01-01

### Added

- Initial placeholder release
