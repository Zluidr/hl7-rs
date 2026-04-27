//! # hl7-mllp
//!
//! Transport-agnostic MLLP (Minimal Lower Layer Protocol) framing for HL7 v2 messages.
//!
//! MLLP is the standard transport envelope used by HL7 v2 over TCP/IP. This crate
//! provides pure framing logic — encoding and decoding MLLP frames — without coupling
//! to any specific async runtime, I/O library, or transport mechanism.
//!
//! ## What is MLLP?
//!
//! MLLP (Minimal Lower Layer Protocol) wraps HL7 v2 messages with simple byte delimiters
//! for reliable streaming over TCP. It is defined in HL7 v2.5.1 Appendix C.
//!
//! ## MLLP Frame Format
//!
//! An MLLP frame wraps an HL7 message with 3 special bytes:
//!
//! ```text
//! +--------+----------------------------+--------+--------+
//! |  VT    |      HL7 message bytes     |   FS   |   CR   |
//! | 0x0B   |        (variable)            | 0x1C   | 0x0D   |
//! +--------+----------------------------+--------+--------+
//!    ↑                                    ↑       ↑
//!    Start of Block                       End of  Line
//!    (Vertical Tab)                       Block   Terminator
//!                                         (File   (Carriage
//!                                         Sep.)   Return)
//! ```
//!
//! - **VT (0x0B)**: Start of block marker. Every frame MUST begin with this byte.
//! - **FS (0x1C)**: End of block marker. Every frame MUST end with FS followed by CR.
//! - **CR (0x0D)**: Carriage return terminator. Required after FS.
//!
//! The payload between VT and FS-CR is the raw HL7 message (typically ER7-encoded).
//!
//! ## Design Philosophy
//!
//! This crate provides three main abstractions:
//!
//! - **[`MllpFrame`]**: Stateless encode/decode operations. Use for simple one-shot framing.
//! - **[`MllpFramer`]**: Stateful streaming accumulator. Use for network I/O where data
//!   arrives in chunks.
//! - **[`MllpTransport`]**: Trait for implementing transports (TCP, serial, etc.).
//! - **[`AsyncMllpTransport`]**: Async variant of the transport trait (requires `async` feature).
//!
//! All operations are:
//! - **Zero-allocation where possible**: `decode()` returns a slice into the original buffer.
//! - **No async/await in core**: Works with blocking or async code equally well.
//! - **Optional async support**: Enable the `async` feature for async transport trait.
//! - **No I/O opinions**: You bring your own sockets/streams.
//!
//! ## Quick Start
//!
//! ### Encode a message for sending
//!
//! ```rust
//! use hl7_mllp::MllpFrame;
//!
//! let raw_hl7 = b"MSH|^~\\&|SendApp|SendFac|20240101120000||ORU^R01|12345|P|2.5\r";
//! let framed = MllpFrame::encode(raw_hl7);
//! // framed now contains: VT + raw_hl7 + FS + CR
//! // Send `framed` over your TCP socket...
//! ```
//!
//! ### Decode a received frame
//!
//! ```rust
//! use hl7_mllp::MllpFrame;
//! use bytes::Bytes;
//!
//! // Received from TCP socket...
//! let framed: Bytes = MllpFrame::encode(b"MSH|^~\\&|...");
//!
//! // decode() returns a slice into the original buffer (zero copy)
//! let decoded = MllpFrame::decode(&framed).unwrap();
//! assert_eq!(decoded, b"MSH|^~\\&|...");
//! ```
//!
//! ### Streaming with MllpFramer
//!
//! ```rust
//! use hl7_mllp::MllpFramer;
//!
//! let mut framer = MllpFramer::new();
//!
//! // Data arrives in chunks from TCP...
//! framer.push(b"\x0BMSH|^~\\&|");
//! framer.push(b"test message\x1C\x0D");
//!
//! // Extract complete frame when available
//! if let Some(frame) = framer.next_frame() {
//!     // Process the complete frame
//!     println!("Received {} bytes", frame.len());
//! }
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `std` | **yes** | Enables `std`-only traits: `MllpTransport`, `AsyncMllpTransport`, and `std::error::Error` impl. |
//! | `noncompliance` | no | Tolerates extra bytes before `VT` and missing final `CR`. |
//! | `timestamps` | no | Adds `chrono`-based timestamps to ACK/NACK generation. Requires `std`. |
//! | `async` | no | Enables `AsyncMllpTransport` trait. Requires `std` and `tokio`. |
//!
//! ### `no_std` Support
//!
//! Disable default features to build without `std`:
//!
//! ```toml
//! [dependencies]
//! hl7-mllp = { version = "0.1", default-features = false }
//! ```
//!
//! **Important**: `no_std` mode still requires an allocator (`alloc` crate).
//! `MllpError::InvalidFrame` carries an owned `String`, and `MllpFramer`
//! uses `BytesMut` internally. If your target has no allocator, this crate
//! is not suitable.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, string::ToString, vec::Vec};

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
    /// Invalid frame format with detailed reason.
    InvalidFrame {
        /// Detailed explanation of why the frame is invalid.
        reason: String,
    },
}

impl core::fmt::Display for MllpError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingStartByte => {
                write!(
                    f,
                    "MLLP frame missing VT start byte (expected 0x0B at position 0)"
                )
            }
            Self::MissingEndSequence => {
                write!(
                    f,
                    "MLLP frame missing FS+CR end sequence (expected 0x1C 0x0D)"
                )
            }
            Self::EmptyPayload => {
                write!(f, "MLLP frame contains no HL7 payload between delimiters")
            }
            Self::Incomplete => {
                write!(
                    f,
                    "Buffer too short for complete MLLP frame (need at least 4 bytes: VT + payload + FS + CR)"
                )
            }
            Self::InvalidFrame { reason } => {
                write!(f, "Invalid MLLP frame: {reason}")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MllpError {}

#[cfg(feature = "std")]
impl From<MllpError> for std::io::Error {
    fn from(err: MllpError) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, err)
    }
}

/// MLLP frame encoder and decoder.
///
/// This struct contains only associated functions — there is no state.
/// It operates purely on byte slices and [`Bytes`].
pub struct MllpFrame;

impl MllpFrame {
    /// Wrap a raw HL7 message payload in an MLLP frame.
    ///
    /// # Output Layout
    ///
    /// The returned [`Bytes`] contains exactly:
    ///
    /// | Byte(s) | Value | Description |
    /// |---------|-------|-------------|
    /// | 0       | 0x0B  | VT (Vertical Tab) - start of block |
    /// | 1..n    | payload | Raw HL7 message bytes (n = payload.len()) |
    /// | n+1     | 0x1C  | FS (File Separator) - end of block |
    /// | n+2     | 0x0D  | CR (Carriage Return) - terminator |
    ///
    /// Total length: `payload.len() + 3` bytes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use hl7_mllp::{MllpFrame, VT, FS, CR};
    ///
    /// let payload = b"MSH|^~\\&|test";
    /// let frame = MllpFrame::encode(payload);
    ///
    /// assert_eq!(frame[0], VT);
    /// assert_eq!(&frame[1..14], payload);
    /// assert_eq!(frame[14], FS);
    /// assert_eq!(frame[15], CR);
    /// ```
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
    /// # Zero-Copy Guarantee
    ///
    /// This method returns a slice `&[u8]` that points into the original `buf`.
    /// No data is copied — this is O(1) regardless of payload size.
    ///
    /// The returned slice has the same lifetime as the input buffer. If you need
    /// an owned copy, call `.to_vec()` on the result.
    ///
    /// # Validation
    ///
    /// This function validates:
    /// - Buffer length ≥ 4 bytes (VT + at least 1 payload byte + FS + CR)
    /// - First byte is VT (0x0B)
    /// - Last two bytes are FS (0x1C) + CR (0x0D)
    /// - Payload is not empty
    ///
    /// # Errors
    ///
    /// Returns [`MllpError`] variants:
    /// - [`Incomplete`](MllpError::Incomplete) if buffer is too short
    /// - [`MissingStartByte`](MllpError::MissingStartByte) if first byte is not VT
    /// - [`MissingEndSequence`](MllpError::MissingEndSequence) if FS+CR not found at end
    /// - [`EmptyPayload`](MllpError::EmptyPayload) if payload length is 0
    ///
    /// # Example
    ///
    /// ```rust
    /// use hl7_mllp::MllpFrame;
    ///
    /// let frame = MllpFrame::encode(b"MSH|test");
    /// let payload = MllpFrame::decode(&frame).unwrap();
    ///
    /// // payload is a slice into frame — zero copy
    /// assert_eq!(payload, b"MSH|test");
    /// ```
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
    /// # Returns
    /// Returns `Some(String)` with the ACK payload, or `None` if `message_control_id` is empty.
    pub fn build_ack(message_control_id: &str, accepting: bool) -> Option<String> {
        if message_control_id.is_empty() {
            return None;
        }
        let code = if accepting { "AA" } else { "AE" };
        let timestamp = chrono_now_str();
        // Generate a unique ACK control ID (ACK + timestamp + original ID)
        let ack_control_id = format!("ACK{}{}", &timestamp, message_control_id);
        Some(format!(
            "MSH|^~\\&|||||{}||ACK|{}|P|2.3.1\rMSA|{}|{}",
            timestamp, ack_control_id, code, message_control_id,
        ))
    }

    /// Build a minimal HL7 NACK message payload with error details (not MLLP-framed).
    ///
    /// `message_control_id` should be the message control ID from the original MSH-10.
    /// `error_code` should be an HL7 error code (e.g., "101", "102").
    /// `error_text` should be a human-readable error description.
    ///
    /// # Returns
    /// Returns `Some(String)` with the NACK payload, or `None` if `message_control_id` is empty.
    pub fn build_nack(
        message_control_id: &str,
        error_code: &str,
        error_text: &str,
    ) -> Option<String> {
        if message_control_id.is_empty() {
            return None;
        }
        let timestamp = chrono_now_str();
        // Generate a unique NACK control ID (NACK + timestamp + original ID)
        let nack_control_id = format!("NACK{}{}", &timestamp, message_control_id);
        // Escape any pipe characters in error text to prevent breaking HL7 field structure
        let escaped_text = error_text.replace('|', "\\F\\");
        // Per HL7 spec: MSA-1 = AR, MSA-2 = original control ID, MSA-3 = error text
        // Error code should be in separate ERR segment (not in MSA)
        Some(format!(
            "MSH|^~\\&|||||{}||ACK|{}|P|2.3.1\rMSA|AR|{}|{}: {} - {}",
            timestamp, nack_control_id, message_control_id, error_code, error_code, escaped_text,
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

/// Stateful streaming frame accumulator for MLLP protocol.
///
/// `MllpFramer` is designed for network I/O where data arrives in chunks.
/// It maintains an internal [`BytesMut`] buffer and provides incremental
/// frame extraction as complete MLLP frames become available.
///
/// # Streaming Usage Pattern
///
/// The typical streaming workflow:
/// 1. Create a framer with `MllpFramer::new()`
/// 2. In a loop, read bytes from your socket/stream
/// 3. Push received bytes into the framer with `push()`
/// 4. Repeatedly call `next_frame()` to extract all complete frames
/// 5. Process each frame, then continue reading
///
/// # Handling Partial Frames
///
/// If `next_frame()` returns `None`, the buffer contains a partial frame
/// (incomplete). Keep the framer alive and push more bytes — the partial
/// data is preserved for the next call.
///
/// # Thread Safety
///
/// `MllpFramer` is not `Sync` — it cannot be shared between threads.
/// It is `Clone` (cheap, since [`BytesMut`] uses ref-counting), so you
/// can clone it if needed for single-threaded scenarios.
///
/// # Example: TCP Streaming
///
/// ```rust
/// use hl7_mllp::MllpFramer;
///
/// let mut framer = MllpFramer::new();
///
/// // Simulate receiving data in chunks from TCP
/// framer.push(b"\x0BMSH|^~\\&|");          // First chunk
/// framer.push(b"partial data...");          // More data
/// framer.push(b"\x1C\x0D\x0BMSH|second");   // Complete frame + partial
///
/// // Extract first complete frame
/// let frame1 = framer.next_frame().unwrap();
/// assert!(frame1.starts_with(&[0x0B]));      // Starts with VT
/// assert!(frame1.ends_with(&[0x1C, 0x0D]));  // Ends with FS+CR
///
/// // No more complete frames yet
/// assert!(framer.next_frame().is_none());
///
/// // Push remaining bytes to complete second frame
/// framer.push(b"|more\x1C\x0D");
/// let frame2 = framer.next_frame().unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct MllpFramer {
    buffer: BytesMut,
}

impl MllpFramer {
    /// Create a new empty framer.
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new(),
        }
    }

    /// Create a new framer with specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
        }
    }

    /// Append bytes to the internal buffer.
    pub fn push(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// Extract the next complete frame if available.
    ///
    /// Returns `Some(Vec<u8>)` with the complete frame (including delimiters),
    /// or `None` if no complete frame is available yet.
    ///
    /// The returned frame is removed from the internal buffer.
    pub fn next_frame(&mut self) -> Option<Vec<u8>> {
        // Find the end of the first complete frame
        let frame_end = MllpFrame::find_frame_end(&self.buffer)?;

        // Extract the frame bytes
        let frame = self.buffer.split_to(frame_end).to_vec();
        Some(frame)
    }

    /// Returns true if the internal buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns the number of bytes in the internal buffer.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Clear the internal buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Returns a mutable reference to the internal buffer for direct I/O.
    ///
    /// This is useful for cancellation-safe async reads. Using
    /// [`tokio::io::AsyncReadExt::read_buf`] with this buffer ensures that
    /// bytes are atomically appended without risk of loss if the future
    /// is cancelled mid-read.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use hl7_mllp::MllpFramer;
    /// use tokio::io::AsyncReadExt;
    ///
    /// async fn read_frame<R: AsyncReadExt>(framer: &mut MllpFramer, reader: &mut R) {
    ///     // read_buf atomically appends to the framer's buffer
    ///     let n = reader.read_buf(framer.read_buf()).await.unwrap();
    ///     println!("Appended {} bytes", n);
    /// }
    /// ```
    pub fn read_buf(&mut self) -> &mut BytesMut {
        &mut self.buffer
    }
}

impl Default for MllpFramer {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can act as an MLLP byte-stream transport.
///
/// Implement this trait for TCP streams, serial ports, in-memory buffers,
/// or any other byte-stream source. This crate provides no concrete
/// implementation — that is intentionally left to consumers.
///
/// # Implementation Contract
///
/// ## Thread Safety
/// - The transport is **not required to be `Sync`**. Each transport instance
///   should be used from a single thread, or synchronized externally.
/// - The transport **should be `Send`** if you need to move it between threads.
///
/// ## Error Handling
/// - `read_frame` should return an error only for I/O failures (broken socket,
///   timeout, etc.), not for malformed MLLP frames.
/// - Malformed frames should be handled by the caller after successful read.
/// - `write_frame` should complete the write or return an error — partial
///   writes are considered failures.
///
/// ## Frame Boundaries
/// - `read_frame` must return **exactly one complete MLLP frame** per call.
/// - It should accumulate bytes internally until `FS+CR` is found.
/// - Consider using [`MllpFramer`] for the accumulation logic.
///
/// ## Blocking Behavior
/// - `read_frame` may block until a complete frame is available.
/// - Non-blocking transports should use an async runtime and return
///   `WouldBlock` errors appropriately.
///
/// # Example Implementation
///
/// ```rust,ignore
/// use hl7_mllp::{MllpTransport, MllpFramer, MllpFrame};
/// use std::net::TcpStream;
/// use std::io::{self, Read, Write};
///
/// pub struct TcpMllpTransport {
///     stream: TcpStream,
///     framer: MllpFramer,
/// }
///
/// impl MllpTransport for TcpMllpTransport {
///     type Error = io::Error;
///
///     fn read_frame(&mut self) -> Result<Vec<u8>, Self::Error> {
///         let mut buf = [0u8; 1024];
///         loop {
///             // Try to extract a complete frame first
///             if let Some(frame) = self.framer.next_frame() {
///                 return Ok(frame);
///             }
///             // Read more bytes from TCP
///             let n = self.stream.read(&mut buf)?;
///             if n == 0 {
///                 return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "connection closed"));
///             }
///             self.framer.push(&buf[..n]);
///         }
///     }
///
///     fn write_frame(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
///         self.stream.write_all(frame)
///     }
/// }
/// ```
#[cfg(feature = "std")]
pub trait MllpTransport {
    /// The error type returned by this transport.
    type Error: std::error::Error;

    /// Read the next complete MLLP-framed message from the transport.
    ///
    /// Implementations are responsible for accumulating bytes until a
    /// complete frame is available. Use [`MllpFrame::find_frame_end`]
    /// or [`MllpFramer`] as the completion signal.
    fn read_frame(&mut self) -> Result<Vec<u8>, Self::Error>;

    /// Write an MLLP-framed message to the transport.
    ///
    /// The frame should be a complete MLLP frame (including VT and FS+CR).
    /// Implementations must ensure the entire frame is written.
    fn write_frame(&mut self, frame: &[u8]) -> Result<(), Self::Error>;
}

/// Async trait for types that can act as an MLLP byte-stream transport.
///
/// This is the async equivalent of [`MllpTransport`]. Implement this for
/// async TCP streams, TLS connections, or any other async byte-stream source.
///
/// # Feature Flag
///
/// This trait is only available when the `async` feature is enabled:
///
/// ```toml
/// [dependencies]
/// hl7-mllp = { version = "0.1", features = ["async"] }
/// ```
///
/// # Implementation Contract
///
/// Same contract as [`MllpTransport`], but with async methods:
///
/// ## Cancellation Safety
/// - `read_frame` should be cancellation-safe (leave the transport in a
///   consistent state if the future is dropped mid-read)
/// - Consider using [`tokio::io::AsyncReadExt::read_buf`] for cancellation safety
///
/// ## Error Handling
/// - Return transport-level errors (broken connection, timeout, etc.)
/// - MLLP framing errors should be handled by the caller after successful read
///
/// ## Frame Boundaries
/// - Return exactly one complete MLLP frame per call
/// - Accumulate bytes internally using [`MllpFramer`] until `FS+CR` is found
///
/// # Example Implementation
///
/// ```rust,ignore
/// use hl7_mllp::AsyncMllpTransport;
/// use tokio::io::{AsyncReadExt, AsyncWriteExt};
/// use tokio::net::TcpStream;
///
/// pub struct AsyncTcpMllpTransport {
///     stream: TcpStream,
///     framer: MllpFramer,
/// }
///
/// impl AsyncMllpTransport for AsyncTcpMllpTransport {
///     type Error = std::io::Error;
///
///     async fn read_frame(&mut self) -> Result<Vec<u8>, Self::Error> {
///         loop {
///             if let Some(frame) = self.framer.next_frame() {
///                 return Ok(frame);
///             }
///             // Use read_buf for cancellation safety - bytes go directly
///             // into the framer's buffer, never lost if the future is dropped
///             let n = self.stream.read_buf(self.framer.read_buf()).await?;
///             if n == 0 {
///                 return Err(std::io::Error::new(
///                     std::io::ErrorKind::UnexpectedEof,
///                     "connection closed",
///                 ));
///             }
///         }
///     }
///
///     async fn write_frame(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
///         self.stream.write_all(frame).await
///     }
/// }
/// ```
#[cfg(feature = "async")]
pub trait AsyncMllpTransport {
    /// The error type returned by this transport.
    type Error: std::error::Error + Send;

    /// Read the next complete MLLP-framed message from the transport.
    ///
    /// Implementations are responsible for accumulating bytes until a
    /// complete frame is available. Use [`MllpFrame::find_frame_end`]
    /// or [`MllpFramer`] as the completion signal.
    ///
    /// # Cancellation Safety
    ///
    /// This method should be cancellation-safe. If the future is dropped
    /// before completion, the transport should remain in a consistent state
    /// such that a subsequent call will continue from where it left off.
    fn read_frame(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, Self::Error>> + Send;

    /// Write an MLLP-framed message to the transport.
    ///
    /// The frame should be a complete MLLP frame (including VT and FS+CR).
    /// Implementations must ensure the entire frame is written.
    fn write_frame(
        &mut self,
        frame: &[u8],
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
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
        // MSA-1 = AA, MSA-2 = original control ID
        assert!(ack.contains("MSA|AA|MSG001"));
    }

    #[test]
    fn build_ack_creates_ae_for_reject() {
        let ack = MllpFrame::build_ack("MSG001", false).unwrap();
        // MSA-1 = AE, MSA-2 = original control ID
        assert!(ack.contains("MSA|AE|MSG001"));
    }

    #[test]
    fn build_ack_has_unique_control_id() {
        let ack = MllpFrame::build_ack("MSG001", true).unwrap();
        // MSH-10 should contain ACK prefix + timestamp + original ID
        assert!(ack.contains("||ACK|ACK"));
        assert!(ack.contains("MSG001|P|2.3.1"));
    }

    #[test]
    fn build_nack_validates_empty_control_id() {
        assert!(MllpFrame::build_nack("", "101", "Error").is_none());
    }

    #[test]
    fn build_nack_creates_ar_with_error_details() {
        let nack = MllpFrame::build_nack("MSG001", "101", "Invalid message").unwrap();
        // MSA-1 = AR, MSA-2 = original control ID, MSA-3 = error text with code
        assert!(nack.contains("MSA|AR|MSG001|101: 101 - Invalid message"));
    }

    #[test]
    fn build_nack_contains_ack_msh() {
        let nack = MllpFrame::build_nack("MSG001", "102", "Parse error").unwrap();
        // Should have MSH with ACK message type and unique control ID
        assert!(nack.starts_with("MSH|^~\\&|||||"));
        assert!(nack.contains("||ACK|NACK")); // NACK prefix for unique ID
    }

    #[test]
    fn build_nack_escapes_pipe_in_error_text() {
        let nack = MllpFrame::build_nack("MSG001", "101", "Error|with|pipes").unwrap();
        // Pipe characters should be escaped as \F\
        assert!(nack.contains("Error\\F\\with\\F\\pipes"));
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

        // Verify MSA-2 contains original control ID
        let msa_seg = msa.unwrap();
        let msa_2 = msa_seg.raw_fields().get(1);
        assert_eq!(msa_2, Some(&"MSG12345"));
    }

    // T1.2 — Round-trip NACK parse test
    #[test]
    fn nack_roundtrip_parse() {
        use hl7_v2::Hl7Message;

        let nack_str = MllpFrame::build_nack("MSG999", "102", "Processing failed").unwrap();
        let nack_bytes = nack_str.as_bytes();

        // Parse the NACK using hl7-v2 crate
        let parsed = Hl7Message::parse(nack_bytes);
        assert!(
            parsed.is_ok(),
            "NACK should be valid HL7 that hl7-v2 can parse"
        );

        let msg = parsed.unwrap();
        // Verify MSH segment exists
        let msh = msg.segment("MSH");
        assert!(msh.is_some(), "NACK should have MSH segment");

        // Verify MSA segment exists
        let msa = msg.segment("MSA");
        assert!(msa.is_some(), "NACK should have MSA segment");

        // Verify MSA-1 = AR (Application Reject)
        let msa_seg = msa.unwrap();
        let msa_1 = msa_seg.raw_fields().first();
        assert_eq!(msa_1, Some(&"AR"));

        // Verify MSA-2 contains original control ID
        let msa_2 = msa_seg.raw_fields().get(1);
        assert_eq!(msa_2, Some(&"MSG999"));
    }

    // T1.3 — Streaming support tests
    #[test]
    fn framer_push_single_bytes_and_recover_frame() {
        let mut framer = MllpFramer::new();
        let frame = MllpFrame::encode(b"MSH|test");

        // Push bytes one at a time
        for byte in &frame {
            assert!(framer.next_frame().is_none());
            framer.push(&[*byte]);
        }

        // Now we should have a complete frame
        let recovered = framer.next_frame().unwrap();
        assert_eq!(recovered, frame.to_vec());

        // Framer should be empty now
        assert!(framer.is_empty());
    }

    #[test]
    fn framer_push_two_frames_at_once() {
        let mut framer = MllpFramer::new();
        let frame1 = MllpFrame::encode(b"MSH|first");
        let frame2 = MllpFrame::encode(b"MSH|second");

        // Push both frames in one call
        let combined = [&frame1[..], &frame2[..]].concat();
        framer.push(&combined);

        // Should recover first frame
        let recovered1 = framer.next_frame().unwrap();
        assert_eq!(recovered1, frame1.to_vec());

        // Should recover second frame
        let recovered2 = framer.next_frame().unwrap();
        assert_eq!(recovered2, frame2.to_vec());

        // No more frames
        assert!(framer.next_frame().is_none());
        assert!(framer.is_empty());
    }

    #[test]
    fn framer_is_empty_and_len() {
        let mut framer = MllpFramer::new();
        assert!(framer.is_empty());
        assert_eq!(framer.len(), 0);

        framer.push(b"\x0Btest");
        assert!(!framer.is_empty());
        assert_eq!(framer.len(), 5); // VT + "test" = 5 bytes

        framer.clear();
        assert!(framer.is_empty());
        assert_eq!(framer.len(), 0);
    }

    #[test]
    fn framer_with_capacity() {
        let framer = MllpFramer::with_capacity(1024);
        assert!(framer.is_empty());
    }

    #[test]
    fn framer_default() {
        let framer: MllpFramer = Default::default();
        assert!(framer.is_empty());
    }

    #[test]
    fn framer_partial_frame_no_complete() {
        let mut framer = MllpFramer::new();
        // Incomplete frame (no FS+CR)
        framer.push(b"\x0Bpartial_data");

        // Should not return a complete frame
        assert!(framer.next_frame().is_none());
        assert!(!framer.is_empty());
    }

    #[test]
    fn framer_preserves_remaining_bytes() {
        let mut framer = MllpFramer::new();
        let frame1 = MllpFrame::encode(b"MSH|first");
        let partial = b"\x0BMSH|partial_no_end";

        // Push complete frame + partial frame
        let combined = [&frame1[..], &partial[..]].concat();
        framer.push(&combined);

        // Extract complete frame
        let recovered = framer.next_frame().unwrap();
        assert_eq!(recovered, frame1.to_vec());

        // Partial frame should remain in buffer
        assert!(!framer.is_empty());
        assert_eq!(framer.len(), partial.len());

        // No complete frame yet
        assert!(framer.next_frame().is_none());
    }

    // T1.6 — Additional tests
    #[test]
    fn encode_decode_roundtrip_unicode() {
        // Test with Unicode payload (UTF-8 encoded)
        let unicode_payload = "MSH|^~\\&|Test|Facility|20240101120000||ORU^R01|12345|P|2.5\rPID|1||P001||Doe^John^José||19800101|M".as_bytes();
        let framed = MllpFrame::encode(unicode_payload);
        let decoded = MllpFrame::decode(&framed).unwrap();
        assert_eq!(decoded, unicode_payload);
    }

    #[test]
    fn decode_minimum_length_valid_frame() {
        // Minimum valid frame: VT + 1 byte payload + FS + CR = 4 bytes
        let min_frame = [VT, b'X', FS, CR];
        let decoded = MllpFrame::decode(&min_frame).unwrap();
        assert_eq!(decoded, b"X");
    }

    #[test]
    fn find_frame_end_exactly_one_frame() {
        // Buffer containing exactly one complete frame (no trailing bytes)
        let payload = b"MSH|exact";
        let frame = MllpFrame::encode(payload);

        let end = MllpFrame::find_frame_end(&frame);
        assert_eq!(end, Some(frame.len()));
    }

    #[test]
    fn find_frame_end_empty_buffer() {
        // Empty buffer should return None
        assert_eq!(MllpFrame::find_frame_end(b""), None);
    }

    #[test]
    fn find_frame_end_no_vt() {
        // Buffer without VT start byte should return None
        assert_eq!(MllpFrame::find_frame_end(b"no_vt_here"), None);
    }

    #[test]
    fn framer_push_pop_streaming() {
        // Test push/pop streaming pattern
        let mut framer = MllpFramer::new();
        let frames = vec![
            MllpFrame::encode(b"MSH|msg1"),
            MllpFrame::encode(b"MSH|msg2"),
            MllpFrame::encode(b"MSH|msg3"),
        ];

        // Push all frames at once
        let combined: Vec<u8> = frames.iter().flat_map(|f| f.to_vec()).collect();
        framer.push(&combined);

        // Pop frames one by one
        for (i, expected) in frames.iter().enumerate() {
            let actual = framer.next_frame().unwrap();
            assert_eq!(actual, expected.to_vec(), "Frame {} mismatch", i);
        }

        // No more frames
        assert!(framer.next_frame().is_none());
    }
}
