# hl7-rs

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![CI](https://github.com/Zluidr/hl7-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Zluidr/hl7-rs/actions/workflows/ci.yml)

**Transport-agnostic HL7 v2 and FHIR R4 crates for Rust — with Indonesian SATUSEHAT support.**

A composable, layered family of crates for healthcare data integration. Each crate has one job and no opinion on the others' runtime, transport, or framework choices.

---

## Crates

| Crate | crates.io | Description |
|---|---|---|
| [`hl7-mllp`](crates/hl7-mllp) | [![](https://img.shields.io/crates/v/hl7-mllp.svg)](https://crates.io/crates/hl7-mllp) | Transport-agnostic MLLP framing |
| [`hl7-v2`](crates/hl7-v2) | [![](https://img.shields.io/crates/v/hl7-v2.svg)](https://crates.io/crates/hl7-v2) | Zero-dependency HL7 v2 parser |
| [`hl7-mindray`](crates/hl7-mindray) | [![](https://img.shields.io/crates/v/hl7-mindray.svg)](https://crates.io/crates/hl7-mindray) | Mindray device HL7 field mappings |
| [`hl7-arrow`](crates/hl7-arrow) *(planned, v0.1)* | — | Apache Arrow RecordBatch emission for cross-language HL7 v2 pipelines |
| [`fhir-r4`](crates/fhir-r4) | [![](https://img.shields.io/crates/v/fhir-r4.svg)](https://crates.io/crates/fhir-r4) | FHIR R4 resource types and builders |
| [`satusehat`](crates/satusehat) | [![](https://img.shields.io/crates/v/satusehat.svg)](https://crates.io/crates/satusehat) | Indonesian SATUSEHAT FHIR profile |

---

## Design Philosophy

```
hl7-mllp        — frame bytes in/out, nothing else
hl7-v2          — parse bytes → typed AST, nothing else
hl7-mindray     — map Mindray codes → VitalSign enum
hl7-arrow       — emit parsed AST → Apache Arrow RecordBatch   [v0.1, planned]
fhir-r4         — FHIR R4 resource structs + builders
satusehat       — Indonesian profile extensions + API config
```

No crate has opinions on async runtimes, I/O frameworks, or application architecture. They compose cleanly with whatever stack you bring.

---

## Quick Start

```toml
[dependencies]
hl7-mllp    = "0.0.1"
hl7-v2      = "0.0.1"
hl7-mindray = "0.0.1"
fhir-r4     = "0.0.1"
satusehat   = "0.0.1"

# v0.1 (planned):
# hl7-arrow = "0.1"
```

```rust
use hl7_mllp::MllpFrame;
use hl7_v2::Hl7Message;
use hl7_mindray::MindrayOru;
use fhir_r4::observation::{ObservationBuilder, ObservationStatus};
use satusehat::{SatuSehatConfig, SatuSehatEnv};
use satusehat::observation::SatuSehatObservation;

// 1. Strip MLLP framing from raw TCP bytes
let payload = MllpFrame::decode(&tcp_bytes)?;

// 2. Parse HL7 v2 message
let msg = Hl7Message::parse(payload)?;

// 3. Extract Mindray vital signs
let oru = MindrayOru::from_message(&msg)?;

// 4. Build FHIR R4 Observation
let obs = ObservationBuilder::new()
    .status(ObservationStatus::Final)
    .loinc_code("59408-5", "Oxygen saturation")
    .value_quantity(oru.spo2().unwrap_or(0.0), "%")
    .patient_reference("Patient/P001")
    .build();

// 5. Wrap with SATUSEHAT profile
let config = SatuSehatConfig {
    env: SatuSehatEnv::Sandbox,
    client_id: std::env::var("SATUSEHAT_CLIENT_ID").unwrap(),
    client_secret: std::env::var("SATUSEHAT_CLIENT_SECRET").unwrap(),
    organization_id: std::env::var("SATUSEHAT_ORG_ID").unwrap(),
};

let ss_obs = SatuSehatObservation::from_observation(obs, &config);
let json = ss_obs.to_json()?;
// POST json to SATUSEHAT FHIR endpoint
```

---

## Dependency Graph

```
satusehat
  └── fhir-r4
        └── serde, serde_json

hl7-mindray
  └── hl7-v2          (zero deps)

hl7-arrow  [v0.1, planned]
  └── hl7-v2          (zero deps)
  └── arrow-rs        (Apache-2.0, cross-language columnar format)

hl7-mllp              (bytes only)
```

---

## Cross-language integration via Apache Arrow *(v0.1, planned)*

HL7 v2 parsing is a natural fit for Rust: length-prefixed framing, strict segment structure, bounded memory requirements, predictable latency. Downstream processing — FHIR R4 REST handling, analytics, ML inference, web APIs — is often more convenient in Python, TypeScript, R, or Julia. Bridging the two languages with anything other than a typed, language-neutral, zero-copy-friendly format is a productivity and correctness hazard.

`hl7-arrow` emits parsed HL7 v2 messages as Apache Arrow RecordBatches. Arrow is implemented in C++, Python, Java, Go, JavaScript, R, Julia, Rust, and others — so any consumer can read the stream without re-parsing HL7.

```
┌──────────────────────┐                    ┌────────────────────────┐
│   Source             │                    │   Consumer             │
│   HL7 v2 / MLLP      │                    │   Any Arrow-capable    │
│                      │                    │   language / runtime   │
└──────────┬───────────┘                    └───────────┬────────────┘
           │                                            │
           │ TCP/MLLP bytes                             │ RecordBatch
           │                                            │ (typed columns)
           ▼                                            │
┌──────────────────────┐    Apache Arrow IPC            │
│   Rust process       │    (language-neutral,          │
│   hl7-mllp           │     typed, columnar            │
│     → hl7-v2         │     binary format)             │
│     → hl7-arrow      │─────────────────────────────▶  │
└──────────────────────┘                                │
```

**Transport options** (pick what suits your deployment):
- **stdout → stdin** (simplest) — Rust binary piped into a Python script
- **File or directory** — Rust writes `.arrow` files; consumer watches and ingests
- **Unix socket / named pipe** — in-host IPC
- **Arrow Flight** (RPC over gRPC) — cross-host, authenticated, streaming
- **Shared memory** — zero-copy within a single host

`hl7-arrow` produces and consumes RecordBatches; it takes no position on transport. That's your choice.

**Example (when `hl7-arrow` v0.1 lands):**

```rust
// Rust: MLLP listener → stdout as Arrow IPC stream
use hl7_mllp::MllpFrame;
use hl7_v2::Hl7Message;
use hl7_arrow::{RecordBatchWriter, oru_r01_schema};

let schema = oru_r01_schema();
let mut writer = RecordBatchWriter::new(std::io::stdout(), &schema)?;

loop {
    let payload = MllpFrame::decode_from(&mut tcp_stream)?;
    let msg = Hl7Message::parse(payload)?;
    let batch = hl7_arrow::encode_oru_r01(&msg, &schema)?;
    writer.write(&batch)?;
}
```

```python
# Python: read Arrow IPC stream from stdin
import pyarrow as pa
import sys

reader = pa.ipc.open_stream(sys.stdin.buffer)
for batch in reader:
    df = batch.to_pandas()
    # hand off to downstream processing
```

**Schema versioning.** The Arrow schema published by `hl7-arrow` is versioned **separately** from the crate version. Downstream consumers pin against the Arrow schema version; this lets the Rust crate evolve (new message types, performance improvements, internal refactors) without breaking consumers tied to a specific schema layout. See `crates/hl7-arrow/README.md` for the schema version matrix (when published).

---

## Development

```bash
# Build all crates
cargo build --workspace

# Test all crates
cargo test --workspace

# Publish (in dependency order)
cargo publish -p hl7-mllp
cargo publish -p hl7-v2
cargo publish -p hl7-mindray   # after hl7-v2 indexes (~1 min)
cargo publish -p hl7-arrow     # v0.1 planned; after hl7-v2 indexes (~1 min)
cargo publish -p fhir-r4
cargo publish -p satusehat     # after fhir-r4 indexes (~1 min)
```

---

## Status

`0.0.1` — placeholder release. Active development in progress.

**v0.1 scope:** adding `hl7-arrow` crate for Apache Arrow RecordBatch emission. See [`TODO.md`](TODO.md) for the phased task list.

## License

Apache-2.0 — see [LICENSE](LICENSE).
