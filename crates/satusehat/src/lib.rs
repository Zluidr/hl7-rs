//! # satusehat
//!
//! Indonesian SATUSEHAT national health platform — FHIR R4 profiles, API client,
//! and data models for Rust applications.
//!
//! SATUSEHAT is Indonesia's national health data exchange platform developed by
//! the Ministry of Health (Kemenkes RI). Since Permenkes No. 24 Tahun 2022, all
//! hospitals and health facilities in Indonesia are required to integrate their
//! SIMRS with SATUSEHAT using HL7 FHIR R4.
//!
//! ## Features
//!
//! - SATUSEHAT FHIR R4 profile extensions and validation
//! - Indonesian-specific code systems (ICD-10 ID, SNOMED CT ID, KFA drug codes)
//! - OAuth 2.0 client credentials flow for SATUSEHAT API auth
//! - Sandbox and production environment configuration
//! - FHIR Bundle construction for batch submission
//!
//! Enable the `client` feature for the async HTTP client:
//!
//! ```toml
//! satusehat = { version = "0.0.1", features = ["client"] }
//! ```
//!
//! ## SATUSEHAT Environments
//!
//! | Environment | Base URL |
//! |---|---|
//! | Sandbox | `https://api-satusehat-stg.dto.kemkes.go.id` |
//! | Production | `https://api-satusehat.kemkes.go.id` |
//!
//! ## Example
//!
//! ```rust
//! use satusehat::{SatuSehatEnv, SatuSehatConfig};
//! use satusehat::observation::SatuSehatObservation;
//! use fhir_r4::observation::{ObservationBuilder, ObservationStatus};
//!
//! let config = SatuSehatConfig {
//!     env: SatuSehatEnv::Sandbox,
//!     client_id: "your_client_id".to_string(),
//!     client_secret: "your_client_secret".to_string(),
//!     organization_id: "your_org_id".to_string(),
//! };
//!
//! let obs = ObservationBuilder::new()
//!     .status(ObservationStatus::Final)
//!     .loinc_code("59408-5", "Oxygen saturation")
//!     .value_quantity(98.0, "%")
//!     .patient_reference("Patient/P001")
//!     .build();
//!
//! let ss_obs = SatuSehatObservation::from_observation(obs, &config);
//! let json = ss_obs.to_json().unwrap();
//! // POST json to SATUSEHAT FHIR endpoint
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod codes;
pub mod observation;

use serde::{Deserialize, Serialize};

/// SATUSEHAT deployment environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SatuSehatEnv {
    /// Sandbox (development/testing).
    /// Base URL: `https://api-satusehat-stg.dto.kemkes.go.id`
    Sandbox,
    /// Production.
    /// Base URL: `https://api-satusehat.kemkes.go.id`
    Production,
}

impl SatuSehatEnv {
    /// Base URL for the FHIR R4 endpoint.
    pub fn fhir_base_url(&self) -> &'static str {
        match self {
            Self::Sandbox    => "https://api-satusehat-stg.dto.kemkes.go.id/fhir-r4/v1",
            Self::Production => "https://api-satusehat.kemkes.go.id/fhir-r4/v1",
        }
    }

    /// Auth token endpoint.
    pub fn auth_url(&self) -> &'static str {
        match self {
            Self::Sandbox    => "https://api-satusehat-stg.dto.kemkes.go.id/oauth2/v1/accesstoken",
            Self::Production => "https://api-satusehat.kemkes.go.id/oauth2/v1/accesstoken",
        }
    }
}

/// SATUSEHAT API configuration.
#[derive(Debug, Clone)]
pub struct SatuSehatConfig {
    /// Target environment.
    pub env: SatuSehatEnv,
    /// OAuth 2.0 client ID (from Kemenkes developer portal).
    pub client_id: String,
    /// OAuth 2.0 client secret.
    pub client_secret: String,
    /// Organization ID registered with SATUSEHAT.
    pub organization_id: String,
}

/// An OAuth 2.0 access token response from SATUSEHAT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    /// Bearer token value.
    pub access_token: String,
    /// Token type (always "Bearer").
    pub token_type: String,
    /// Expiry in seconds from issuance.
    pub expires_in: u64,
}

/// Errors from SATUSEHAT operations.
#[derive(Debug)]
pub enum SatuSehatError {
    /// JSON serialization/deserialization error.
    Json(serde_json::Error),
    /// A required field was missing.
    MissingField(String),
    /// HTTP client error (only available with `client` feature).
    #[cfg(feature = "client")]
    Http(reqwest::Error),
}

impl std::fmt::Display for SatuSehatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::MissingField(s) => write!(f, "Missing required field: {s}"),
            #[cfg(feature = "client")]
            Self::Http(e) => write!(f, "HTTP error: {e}"),
        }
    }
}

impl std::error::Error for SatuSehatError {}

impl From<serde_json::Error> for SatuSehatError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
