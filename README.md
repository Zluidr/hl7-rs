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
| [`fhir-r4`](crates/fhir-r4) | [![](https://img.shields.io/crates/v/fhir-r4.svg)](https://crates.io/crates/fhir-r4) | FHIR R4 resource types and builders |
| [`satusehat`](crates/satusehat) | [![](https://img.shields.io/crates/v/satusehat.svg)](https://crates.io/crates/satusehat) | Indonesian SATUSEHAT FHIR profile |

---

## Design Philosophy

```
hl7-mllp        — frame bytes in/out, nothing else
hl7-v2          — parse bytes → typed AST, nothing else
hl7-mindray     — map Mindray codes → VitalSign enum
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

hl7-mllp              (bytes only)
```

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
cargo publish -p fhir-r4
cargo publish -p satusehat     # after fhir-r4 indexes (~1 min)
```

---

## Status

`0.0.1` — placeholder release. Active development in progress.

## License

Apache-2.0 — see [LICENSE](LICENSE).
