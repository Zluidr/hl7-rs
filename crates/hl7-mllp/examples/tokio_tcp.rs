//! Async TCP MLLP listener using Tokio
//!
//! This example demonstrates an async TCP server using tokio that accepts
//! MLLP-framed HL7 messages and responds with ACKs.
//!
//! Run with: cargo run --example tokio_tcp --features async

use hl7_mllp::{AsyncMllpTransport, MllpFrame, MllpFramer};
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Async TCP transport implementing `AsyncMllpTransport`.
pub struct AsyncTcpMllpTransport {
    stream: TcpStream,
    framer: MllpFramer,
}

impl AsyncTcpMllpTransport {
    /// Create a new transport from a connected TCP stream.
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            framer: MllpFramer::new(),
        }
    }
}

impl AsyncMllpTransport for AsyncTcpMllpTransport {
    type Error = io::Error;

    async fn read_frame(&mut self) -> Result<Vec<u8>, Self::Error> {
        loop {
            // Try to extract a complete frame first
            if let Some(frame) = self.framer.next_frame() {
                return Ok(frame);
            }

            // Read more bytes directly into the framer's buffer.
            // Using read_buf (not read) is cancellation-safe: if the future
            // is dropped mid-read, the bytes are already in the buffer.
            let n = self.stream.read_buf(self.framer.read_buf()).await?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "connection closed",
                ));
            }
        }
    }

    async fn write_frame(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.stream.write_all(frame).await
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:2575").await?;
    println!("Async MLLP listener on 127.0.0.1:2575");
    println!("Press Ctrl+C to exit");

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Client connected: {addr}");

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, addr.to_string()).await {
                eprintln!("Error handling client {addr}: {e}");
            }
        });
    }
}

async fn handle_client(stream: TcpStream, addr: String) -> io::Result<()> {
    let mut transport = AsyncTcpMllpTransport::new(stream);

    loop {
        // Read next frame
        let frame = match transport.read_frame().await {
            Ok(frame) => frame,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                println!("Client disconnected: {addr}");
                return Ok(());
            }
            Err(e) => return Err(e),
        };

        println!("Received {} bytes from {addr}", frame.len());

        // Decode and process the HL7 message
        match MllpFrame::decode(&frame) {
            Ok(payload) => {
                let payload_str = String::from_utf8_lossy(payload);
                let control_id = extract_control_id(&payload_str);

                println!("  Message control ID: {control_id}");

                // Build ACK response
                let ack = MllpFrame::build_ack(&control_id, true).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "empty control id")
                })?;
                let ack_framed = MllpFrame::encode(ack.as_bytes());

                // Send ACK back
                if let Err(e) = transport.write_frame(&ack_framed).await {
                    eprintln!("  Failed to send ACK: {e}");
                    return Err(e);
                }
                println!("  Sent ACK");
            }
            Err(e) => {
                eprintln!("  Invalid MLLP frame: {e}");

                // Send NACK
                let nack =
                    MllpFrame::build_nack("UNKNOWN", "101", &e.to_string()).ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "empty control id")
                    })?;
                let nack_framed = MllpFrame::encode(nack.as_bytes());
                let _ = transport.write_frame(&nack_framed).await;
            }
        }
    }
}

fn extract_control_id(payload: &str) -> String {
    // Simple extraction of MSH-10 from ER7-encoded message.
    // MSH-10 is the 10th field: MSH|^~\&|f3|f4|f5|f6|f7|f8|f9|f10(MSH-10)|...
    // Split by | gives indices: 0=MSH, 1=^~\&, 2=f3, ..., 10=MSH-10
    payload
        .split('|')
        .nth(10)
        .map(|s| s.split('\r').next().unwrap_or(s).to_string())
        .unwrap_or_else(|| "UNKNOWN".to_string())
}
