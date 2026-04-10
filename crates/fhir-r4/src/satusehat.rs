//! SATUSEHAT profile extensions for FHIR R4
//!
//! This module provides Indonesian SATUSEHAT-specific extensions
//! to standard FHIR R4 resources.

#![cfg(feature = "satusehat")]

/// SATUSEHAT profile marker trait
pub trait SatuSehatProfile {
    /// Returns the profile URL for SATUSEHAT validation
    fn profile_url(&self) -> &'static str;
}
