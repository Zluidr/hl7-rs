# satusehat

[![Crates.io](https://img.shields.io/crates/v/satusehat.svg)](https://crates.io/crates/satusehat)
[![Docs.rs](https://docs.rs/satusehat/badge.svg)](https://docs.rs/satusehat)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](./LICENSE)

**Indonesian SATUSEHAT national health platform — FHIR R4 profiles, API client, and data models.**

Implements the [SATUSEHAT](https://satusehat.kemkes.go.id) FHIR R4 profiles and API specifications for Indonesia's national health interoperability platform. Since Permenkes No. 24 Tahun 2022, all hospitals and health facilities in Indonesia are required to integrate their SIMRS with SATUSEHAT using HL7 FHIR R4.

---

## Design Philosophy

- **FHIR R4 compliant** — compatible with `fhir-r4` crate
- **SATUSEHAT profiles** — Indonesian-specific extensions and validations
- **Optional HTTP client** — `reqwest`-based client behind `client` feature flag
- **Environment configs** — Sandbox, Staging, Production endpoints built-in

## SATUSEHAT Environments

| Environment | Base URL |
|---|---|
| Sandbox | `https://api-satusehat-stg.dto.kemkes.go.id` |
| Production | `https://api-satusehat.kemkes.go.id` |

---

## Usage

```rust
use satusehat::{SatuSehatConfig, SatuSehatEnv};
use satusehat::observation::SatuSehatObservation;
use fhir_r4::observation::{ObservationBuilder, ObservationStatus};

let config = SatuSehatConfig {
    env: SatuSehatEnv::Sandbox,
    client_id: "your_client_id".to_string(),
    client_secret: "your_client_secret".to_string(),
    organization_id: "your_org_id".to_string(),
};

let obs = ObservationBuilder::new()
    .status(ObservationStatus::Final)
    .loinc_code("59408-5", "Oxygen saturation")
    .value_quantity(98.0, "%")
    .patient_reference("Patient/P001")
    .build();

let ss_obs = SatuSehatObservation::from_observation(obs, &config);
let json = ss_obs.to_json().unwrap();
// POST json to SATUSEHAT FHIR endpoint
```

### With HTTP client

```toml
[dependencies]
satusehat = { version = "0.0.1", features = ["client"] }
```

```rust,ignore
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
