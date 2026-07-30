//! Minimal MLLP → Arrow IPC listener (HT-T10.2).
//!
//! Accepts a single TCP connection, decodes each MLLP-framed HL7 v2 ORU^R01
//! message, encodes it to a [`RecordBatch`] via [`hl7_arrow::encode_oru_r01`],
//! and writes it to stdout as an Arrow IPC stream. Its Python companion,
//! `pyarrow_consumer.py`, reads that stream back.
//!
//! Intentionally minimal: single connection, no reconnect logic, no
//! graceful shutdown, no ACK/NACK. Demonstrates the MLLP → `hl7-v2` →
//! `hl7-arrow` → Arrow IPC pattern — not production code.
//!
//! ## Run
//!
//! ```text
//! cargo run -p hl7-arrow --example mllp_listener > /tmp/out.arrow &
//! # then send an MLLP-framed HL7 v2 message to 127.0.0.1:2575, e.g. via
//! # the fixture at crates/hl7-arrow/tests/fixtures/oru_r01_sample.hl7
//! python3 crates/hl7-arrow/examples/pyarrow_consumer.py < /tmp/out.arrow
//! ```

use std::io::{self, Read, Write};
use std::net::TcpListener;

use arrow::ipc::writer::StreamWriter;
use hl7_arrow::encode_oru_r01;
use hl7_mllp::{MllpFrame, MllpFramer};
use hl7_v2::Hl7Message;

fn main() -> io::Result<()> {
    let addr = "127.0.0.1:2575";
    let listener = TcpListener::bind(addr)?;
    eprintln!("mllp_listener: listening on {addr}");

    let (mut stream, peer) = listener.accept()?;
    eprintln!("mllp_listener: accepted connection from {peer}");

    let mut framer = MllpFramer::new();
    let mut read_buf = [0u8; 4096];
    let mut writer: Option<StreamWriter<io::Stdout>> = None;

    loop {
        let n = stream.read(&mut read_buf)?;
        if n == 0 {
            break; // peer closed the connection
        }
        framer.push(&read_buf[..n]);

        while let Some(frame) = framer.next_frame() {
            let payload = match MllpFrame::decode(&frame) {
                Ok(payload) => payload,
                Err(err) => {
                    eprintln!("mllp_listener: skipping invalid MLLP frame: {err}");
                    continue;
                }
            };

            let message = match Hl7Message::parse(payload) {
                Ok(message) => message,
                Err(err) => {
                    eprintln!("mllp_listener: skipping unparseable HL7 message: {err}");
                    continue;
                }
            };

            let batch = match encode_oru_r01(&message) {
                Ok(batch) => batch,
                Err(err) => {
                    eprintln!("mllp_listener: skipping message outside the ORU^R01 schema: {err}");
                    continue;
                }
            };

            let ipc_writer = writer.get_or_insert_with(|| {
                StreamWriter::try_new(io::stdout(), &batch.schema())
                    .expect("Arrow IPC stream writer on stdout")
            });
            ipc_writer
                .write(&batch)
                .expect("write RecordBatch to Arrow IPC stream");
            ipc_writer.get_mut().flush()?;
        }
    }

    if let Some(mut ipc_writer) = writer {
        ipc_writer
            .finish()
            .expect("finish Arrow IPC stream on stdout");
    }
    Ok(())
}
