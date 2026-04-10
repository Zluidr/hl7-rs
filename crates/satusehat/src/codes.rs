//! Indonesian code systems used in SATUSEHAT.

/// SATUSEHAT / Kemenkes code system URIs.
pub mod systems {
    /// ICD-10 as used in Indonesia (Kemenkes mapping).
    pub const ICD10_ID: &str = "http://hl7.org/fhir/sid/icd-10";
    /// SNOMED CT.
    pub const SNOMED_CT: &str = "http://snomed.info/sct";
    /// LOINC.
    pub const LOINC: &str = "http://loinc.org";
    /// Kemenkes Formularium Nasional (FORNAS) drug codes.
    pub const KFA: &str = "https://api-satusehat.kemkes.go.id/kfa";
    /// SATUSEHAT Organization identifier system.
    pub const SATUSEHAT_ORG: &str = "https://api-satusehat.kemkes.go.id/Organization";
    /// SATUSEHAT Location identifier system.
    pub const SATUSEHAT_LOCATION: &str = "https://api-satusehat.kemkes.go.id/Location";
}

/// Observation category codes required by SATUSEHAT for vital signs.
pub mod vital_sign_categories {
    /// LOINC code for SpO2.
    pub const SPO2: &str = "59408-5";
    /// LOINC code for heart rate.
    pub const HEART_RATE: &str = "8867-4";
    /// LOINC code for respiratory rate.
    pub const RESP_RATE: &str = "9279-1";
    /// LOINC code for body temperature.
    pub const TEMPERATURE: &str = "8310-5";
    /// LOINC code for NIBP systolic.
    pub const NIBP_SYSTOLIC: &str = "8480-6";
    /// LOINC code for NIBP diastolic.
    pub const NIBP_DIASTOLIC: &str = "8462-4";
    /// LOINC code for mean arterial pressure.
    pub const MAP: &str = "8478-0";
}
