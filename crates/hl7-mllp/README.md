# hl7-mllp

[![Crates.io](https://img.shields.io/crates/v/hl7-mllp.svg)](https://crates.io/crates/hl7-mllp)
[![Docs.rs](https://docs.rs/hl7-mllp/badge.svg)](https://docs.rs/hl7-mllp)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**Transport-agnostic MLLP framing for HL7 v2 messages.**

MLLP (Minimal Lower Layer Protocol) is the standard transport envelope for HL7 v2 messages over TCP/IP. This crate provides pure frame encoding and decoding — with no opinion on async runtimes, I/O libraries, or transports.

---

## Design Philosophy

Most existing Rust MLLP crates are tightly coupled to tokio. This crate is not.

- **No tokio dependency** — bring your own transport
- **No async-std dependency** — works in sync, async, and `no_std` contexts
- **[`MllpTransport`] trait** — implement for TCP, serial, UART, in-memory, or anything else
- **Pure framing logic** — encode, decode, find frame boundaries

---

## MLLP Frame Format

```
[VT 0x0B] [HL7 message bytes ...] [FS 0x1C] [CR 0x0D]
```

---

## Usage

```rust
use hl7_mllp::{MllpFrame, MllpError};

// Encode a raw HL7 payload into an MLLP frame
let payload = b"MSH|^~\\&|SendApp|SendFac|...";
let framed = MllpFrame::encode(payload);

// Decode an MLLP frame back to the raw HL7 payload
let decoded = MllpFrame::decode(&framed)?;
assert_eq!(decoded, payload);

// Find frame boundaries in a streaming buffer
if let Some(frame_len) = MllpFrame::find_frame_end(&buffer) {
    let frame = &buffer[..frame_len];
    let payload = MllpFrame::decode(frame)?;
    // process payload...
}
```

### Implementing a custom transport

```rust
use hl7_mllp::MllpTransport;

struct MyTcpTransport { /* ... */ }

impl MllpTransport for MyTcpTransport {
    type Error = std::io::Error;

    fn read_frame(&mut self) -> Result<Vec<u8>, Self::Error> {
        // accumulate bytes until MllpFrame::find_frame_end returns Some
        todo!()
    }

    fn write_frame(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        // write frame bytes to the stream
        todo!()
    }
}
```

---

## Ecosystem

This crate is part of a family of transport-agnostic HL7 and FHIR crates:

| Crate | Purpose |
|---|---|
| [`hl7-mllp`](https://crates.io/crates/hl7-mllp) | MLLP framing (this crate) |
| [`hl7-v2`](https://crates.io/crates/hl7-v2) | HL7 v2 message parser |
| [`hl7-mindray`](https://crates.io/crates/hl7-mindray) | Mindray device HL7 field mappings |
| [`fhir-r4`](https://crates.io/crates/fhir-r4) | FHIR R4 resource types |
| [`satusehat`](https://crates.io/crates/satusehat) | Indonesian SATUSEHAT FHIR profile |

---

## Status

`0.0.1` — initial placeholder. Active development in progress.

## License

Apache-2.0
