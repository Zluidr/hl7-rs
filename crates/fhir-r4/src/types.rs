//! Shared FHIR R4 primitive and complex types.

use serde::{Deserialize, Serialize};

/// A FHIR Reference to another resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    /// Relative or absolute URI reference (e.g., `"Patient/P001"`).
    pub reference: Option<String>,
    /// Display text for the reference.
    pub display: Option<String>,
}

impl Reference {
    /// Create a reference from a resource URI.
    pub fn new(reference: impl Into<String>) -> Self {
        Self {
            reference: Some(reference.into()),
            display: None,
        }
    }
}

/// A FHIR CodeableConcept.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeableConcept {
    /// One or more codings.
    pub coding: Option<Vec<Coding>>,
    /// Plain text representation.
    pub text: Option<String>,
}

impl CodeableConcept {
    /// Create a CodeableConcept from a single coding.
    pub fn from_coding(coding: Coding) -> Self {
        Self {
            coding: Some(vec![coding]),
            text: None,
        }
    }
}

/// A FHIR Coding (system + code + display).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coding {
    /// The code system URI.
    pub system: Option<String>,
    /// The code value.
    pub code: Option<String>,
    /// Human-readable display name.
    pub display: Option<String>,
}

impl Coding {
    /// Create a LOINC coding.
    pub fn loinc(code: impl Into<String>, display: impl Into<String>) -> Self {
        Self {
            system: Some("http://loinc.org".to_string()),
            code: Some(code.into()),
            display: Some(display.into()),
        }
    }

    /// Create a SNOMED CT coding.
    pub fn snomed(code: impl Into<String>, display: impl Into<String>) -> Self {
        Self {
            system: Some("http://snomed.info/sct".to_string()),
            code: Some(code.into()),
            display: Some(display.into()),
        }
    }
}

/// A FHIR Quantity (value + unit).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantity {
    /// Numeric value.
    pub value: Option<f64>,
    /// Unit string (e.g., `"%"`, `"/min"`).
    pub unit: Option<String>,
    /// UCUM system URI.
    pub system: Option<String>,
    /// UCUM code.
    pub code: Option<String>,
}

impl Quantity {
    /// Create a quantity with a value and display unit.
    pub fn new(value: f64, unit: impl Into<String>) -> Self {
        let unit_str = unit.into();
        Self {
            value: Some(value),
            unit: Some(unit_str.clone()),
            system: Some("http://unitsofmeasure.org".to_string()),
            code: Some(unit_str),
        }
    }
}
