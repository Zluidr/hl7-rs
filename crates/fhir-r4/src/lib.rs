//! # fhir-r4
//!
//! FHIR R4 resource types and builders.
//!
//! Provides serde-compatible Rust structs for the FHIR R4 resources most
//! commonly used in device-to-EMR integration pipelines:
//! [`Observation`], [`Patient`], [`Encounter`], and supporting types.
//!
//! ## Design
//!
//! - Focused on the resources needed for vital sign integration
//! - `serde` JSON serialization matching the FHIR R4 spec
//! - Builder pattern for common resource construction
//! - Optional `satusehat` feature for Indonesian SATUSEHAT profile extensions
//!
//! ## Example
//!
//! ```rust
//! use fhir_r4::observation::{Observation, ObservationBuilder, ObservationStatus};
//!
//! let obs = ObservationBuilder::new()
//!     .status(ObservationStatus::Final)
//!     .loinc_code("59408-5", "Oxygen saturation")
//!     .value_quantity(98.0, "%")
//!     .patient_reference("Patient/P001")
//!     .build();
//!
//! let json = serde_json::to_string_pretty(&obs).unwrap();
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod observation;
pub mod patient;
pub mod types;

#[cfg(feature = "satusehat")]
pub mod satusehat;
