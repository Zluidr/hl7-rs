# hl7-v2

[![Crates.io](https://img.shields.io/crates/v/hl7-v2.svg)](https://crates.io/crates/hl7-v2)
[![Docs.rs](https://docs.rs/hl7-v2/badge.svg)](https://docs.rs/hl7-v2)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**Zero-dependency HL7 v2 message parser.**

Parses raw HL7 v2 byte input into a typed message AST. No transport coupling, no runtime dependency, no opinion on I/O.

---

## Design Philosophy

HL7 parsing and message transport are two separate concerns. This crate handles only parsing.

- **Zero dependencies** — not even `serde`
- **No async, no tokio, no runtime** — pure synchronous parsing
- **Composable** — use with [`hl7-mllp`](https://crates.io/crates/hl7-mllp) for TCP, or standalone for file/buffer use
- **Correct encoding character handling** — reads separators from MSH-1/MSH-2, not hardcoded

---

## Usage

```rust
use hl7_v2::Hl7Message;

let raw = b"MSH|^~\\&|App|Fac|App2|Fac2|20240101||ORU^R01|001|P|2.3.1\r\
            OBX|1|NM|59408-5^SpO2^LN||98|%|95-100|N|||F\r\
            OBX|2|NM|8867-4^HR^LN||72|/min|60-100|N|||F";

let msg = Hl7Message::parse(raw)?;

println!("Type: {:?}", msg.message_type());       // Some("ORU^R01")
println!("Version: {:?}", msg.version());          // Some("2.3.1")

for obx in msg.segments("OBX") {
    let loinc = obx.field(3).and_then(|f| f.component(1));
    let value = obx.field(5).map(|f| f.value());
    let unit  = obx.field(6).map(|f| f.value());
    println!("{:?} = {:?} {:?}", loinc, value, unit);
}
```

### Component access

```rust
// OBX-3: "59408-5^SpO2^LN"
let obx = msg.segments("OBX").next().unwrap();
let code   = obx.field(3).and_then(|f| f.component(1)); // "59408-5"
let name   = obx.field(3).and_then(|f| f.component(2)); // "SpO2"
let system = obx.field(3).and_then(|f| f.component(3)); // "LN"
```

---

## Ecosystem

| Crate | Purpose |
|---|---|
| [`hl7-mllp`](https://crates.io/crates/hl7-mllp) | MLLP transport framing |
| [`hl7-v2`](https://crates.io/crates/hl7-v2) | HL7 v2 parser (this crate) |
| [`hl7-mindray`](https://crates.io/crates/hl7-mindray) | Mindray device field mappings |
| [`fhir-r4`](https://crates.io/crates/fhir-r4) | FHIR R4 resource types |
| [`satusehat`](https://crates.io/crates/satusehat) | Indonesian SATUSEHAT FHIR profile |

---

## Status

`0.0.1` — initial placeholder. Active development in progress.  

## License

Apache-2.0
