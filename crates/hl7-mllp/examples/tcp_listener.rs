//! Minimal blocking TCP MLLP listener example
//!
//! This example demonstrates a simple blocking TCP server that accepts
//! MLLP-framed HL7 messages and responds with ACKs.
//!
//! Run with: cargo run --example tcp_listener

use hl7_mllp::{MllpFrame, MllpFramer};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:2575")?;
    println!("MLLP listener on 127.0.0.1:2575");
    println!("Press Ctrl+C to exit");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprintln!("Connection failed: {e}");
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    let addr = stream.peer_addr().unwrap();
    println!("Client connected: {addr}");

    let mut framer = MllpFramer::new();
    let mut buf = [0u8; 4096];

    loop {
        // Try to extract complete frames first
        while let Some(frame) = framer.next_frame() {
            println!("Received {} bytes from {addr}", frame.len());

            // Decode and process the HL7 message
            match MllpFrame::decode(&frame) {
                Ok(payload) => {
                    // Parse MSH-10 for message control ID (simplified)
                    let payload_str = String::from_utf8_lossy(payload);
                    let control_id = extract_control_id(&payload_str);

                    println!("  Message control ID: {control_id}");

                    // Build ACK response
                    let ack = MllpFrame::build_ack(&control_id, true).expect("valid control ID");
                    let ack_framed = MllpFrame::encode(ack.as_bytes());

                    // Send ACK back
                    if let Err(e) = stream.write_all(&ack_framed) {
                        eprintln!("  Failed to send ACK: {e}");
                        return;
                    }
                    println!("  Sent ACK");
                }
                Err(e) => {
                    eprintln!("  Invalid MLLP frame: {e}");

                    // Send NACK
                    let nack = MllpFrame::build_nack("UNKNOWN", "101", &e.to_string())
                        .expect("valid control ID");
                    let nack_framed = MllpFrame::encode(nack.as_bytes());
                    let _ = stream.write_all(&nack_framed);
                }
            }
        }

        // Read more bytes from the socket
        match stream.read(&mut buf) {
            Ok(0) => {
                println!("Client disconnected: {addr}");
                return;
            }
            Ok(n) => {
                framer.push(&buf[..n]);
            }
            Err(e) => {
                eprintln!("Read error from {addr}: {e}");
                return;
            }
        }
    }
}

fn extract_control_id(payload: &str) -> String {
    // Simple extraction of MSH-10 from ER7-encoded message
    // Format: MSH|^~\&|...|...|...|...|...|...|MSG_TYPE|CONTROL_ID|...
    payload
        .split('|')
        .nth(9) // MSH-10 is the 10th field (0-indexed: 9)
        .map(|s| s.split('\r').next().unwrap_or(s).to_string())
        .unwrap_or_else(|| "UNKNOWN".to_string())
}
