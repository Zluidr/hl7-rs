# fhir-r4

[![Crates.io](https://img.shields.io/crates/v/fhir-r4.svg)](https://crates.io/crates/fhir-r4)
[![Docs.rs](https://docs.rs/fhir-r4/badge.svg)](https://docs.rs/fhir-r4)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**FHIR R4 resource types and builders — focused on device integration pipelines.**

Serde-compatible Rust structs for the FHIR R4 resources most commonly used in device-to-EMR integration: `Observation`, `Patient`, `Encounter`, and supporting types.

---

## Design Philosophy

- **Focused scope** — resources relevant to device integration, not a full FHIR server
- **Builder pattern** — ergonomic construction of common resource shapes
- **serde JSON** — direct serialization to FHIR-compliant JSON
- **`satusehat` feature flag** — Indonesian SATUSEHAT profile extensions opt-in

---

## Usage

```rust
use fhir_r4::observation::{ObservationBuilder, ObservationStatus};

let obs = ObservationBuilder::new()
    .status(ObservationStatus::Final)
    .loinc_code("59408-5", "Oxygen saturation")
    .value_quantity(98.0, "%")
    .patient_reference("Patient/P001")
    .effective_datetime("2024-01-01T12:00:00+08:00")
    .build();

let json = serde_json::to_string_pretty(&obs)?;
// POST json to your FHIR server
```

### With SATUSEHAT extensions

```toml
[dependencies]
fhir-r4 = { version = "0.0.1", features = ["satusehat"] }
```

```rust
use fhir_r4::satusehat::SatuSehatObservation;
// SATUSEHAT-specific profile fields and validation
```

---

## Covered Resources

| Resource | Status |
|---|---|
| `Observation` | ✅ Builder + serde |
| `Patient` | 🔄 Stub |
| `Encounter` | 🔄 Planned |
| `Bundle` | 🔄 Planned |
| `Organization` | 🔄 Planned |

---

## Ecosystem

| Crate | Purpose |
|---|---|
| [`hl7-mllp`](https://crates.io/crates/hl7-mllp) | MLLP transport framing |
| [`hl7-v2`](https://crates.io/crates/hl7-v2) | HL7 v2 parser |
| [`hl7-mindray`](https://crates.io/crates/hl7-mindray) | Mindray field mappings |
| [`fhir-r4`](https://crates.io/crates/fhir-r4) | FHIR R4 types (this crate) |
| [`satusehat`](https://crates.io/crates/satusehat) | Indonesian SATUSEHAT FHIR profile |

---

## Status

`0.0.1` — initial placeholder. Active development in progress.  

## License

Apache-2.0
