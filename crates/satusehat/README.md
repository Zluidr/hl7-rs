# satusehat

[![Crates.io](https://img.shields.io/crates/v/satusehat.svg)](https://crates.io/crates/satusehat)
[![Docs.rs](https://docs.rs/satusehat/badge.svg)](https://docs.rs/satusehat)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**Indonesian SATUSEHAT national health platform — FHIR R4 profiles, API client, and data models.**

Implements the [SATUSEHAT](https://satusehat.kemkes.go.id) FHIR R4 profiles and API specifications for Indonesia's national health interoperability platform.

---

## Design Philosophy

- **FHIR R4 compliant** — compatible with `fhir-r4` crate
- **SATUSEHAT profiles** — Indonesian-specific extensions and validations
- **Optional HTTP client** — `reqwest`-based client behind `client` feature flag
- **Environment configs** — Sandbox, Staging, Production endpoints built-in

---

## Usage

```rust
use satusehat::{SatuSehatConfig, SatuSehatEnv};
use satusehat::observation::SatuSehatObservation;
use fhir_r4::observation::{ObservationBuilder, ObservationStatus};

// Build a FHIR Observation
let obs = ObservationBuilder::new()
    .status(ObservationStatus::Final)
    .loinc_code("59408-5", "Oxygen saturation")
    .value_quantity(98.0, "%")
    .patient_reference("Patient/P001")
    .build();

// Wrap with SATUSEHAT profile
let config = SatuSehatConfig {
    env: SatuSehatEnv::Sandbox,
    client_id: std::env::var("SATUSEHAT_CLIENT_ID").unwrap(),
    client_secret: std::env::var("SATUSEHAT_CLIENT_SECRET").unwrap(),
    organization_id: std::env::var("SATUSEHAT_ORG_ID").unwrap(),
};

let ss_obs = SatuSehatObservation::from_observation(obs, &config);
let json = serde_json::to_string_pretty(&ss_obs)?;
```

### With HTTP client

```toml
[dependencies]
satusehat = { version = "0.0.1", features = ["client"] }
```

```rust
use satusehat::client::SatuSehatClient;

let client = SatuSehatClient::new(&config).await?;
let response = client.create_observation(&ss_obs).await?;
```

---

## Ecosystem

| Crate | Purpose |
|---|---|
| [`hl7-mllp`](https://crates.io/crates/hl7-mllp) | MLLP transport framing |
| [`hl7-v2`](https://crates.io/crates/hl7-v2) | HL7 v2 parser |
| [`hl7-mindray`](https://crates.io/crates/hl7-mindray) | Mindray device field mappings |
| [`fhir-r4`](https://crates.io/crates/fhir-r4) | FHIR R4 resource types |
| [`satusehat`](https://crates.io/crates/satusehat) | Indonesian SATUSEHAT FHIR profile (this crate) |

---

## Status

`0.0.1` — initial placeholder. Active development in progress.

## License

Apache-2.0
