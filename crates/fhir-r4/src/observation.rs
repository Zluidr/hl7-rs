//! FHIR R4 Observation resource — the primary resource for vital sign data.

use crate::types::{CodeableConcept, Coding, Quantity, Reference};
use serde::{Deserialize, Serialize};

/// FHIR R4 Observation status codes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservationStatus {
    /// The observation is registered but not yet complete.
    Registered,
    /// The observation is preliminary.
    Preliminary,
    /// The observation is complete and verified.
    Final,
    /// The observation has been modified after being final.
    Amended,
    /// The observation has been corrected.
    Corrected,
    /// The observation has been cancelled.
    Cancelled,
    /// The observation has been entered in error.
    EnteredInError,
    /// The status is unknown.
    Unknown,
}

/// The value of an Observation — one of several FHIR choice types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ObservationValue {
    /// Numeric value with units.
    Quantity(Quantity),
    /// Coded value.
    CodeableConcept(CodeableConcept),
    /// String value.
    String(String),
    /// Boolean value.
    Boolean(bool),
}

/// FHIR R4 Observation resource.
///
/// Used to record vital signs, laboratory results, and other clinical measurements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    /// Always `"Observation"`.
    pub resource_type: String,

    /// Logical identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Observation status — required.
    pub status: ObservationStatus,

    /// Category (e.g., vital-signs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<Vec<CodeableConcept>>,

    /// What was observed (LOINC code).
    pub code: CodeableConcept,

    /// The patient this observation is about.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Reference>,

    /// The clinical encounter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encounter: Option<Reference>,

    /// When the observation was made (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_date_time: Option<String>,

    /// The actual measured value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_quantity: Option<Quantity>,

    /// Reference range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_range: Option<Vec<ObservationReferenceRange>>,
}

/// Reference range for an observation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationReferenceRange {
    /// Lower bound.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low: Option<Quantity>,
    /// Upper bound.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high: Option<Quantity>,
}

/// Builder for [`Observation`] resources.
#[derive(Default)]
pub struct ObservationBuilder {
    id: Option<String>,
    status: Option<ObservationStatus>,
    code: Option<CodeableConcept>,
    subject: Option<Reference>,
    encounter: Option<Reference>,
    effective_date_time: Option<String>,
    value_quantity: Option<Quantity>,
}

impl ObservationBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the observation ID.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the observation status.
    pub fn status(mut self, status: ObservationStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Set a LOINC code for this observation.
    pub fn loinc_code(mut self, code: impl Into<String>, display: impl Into<String>) -> Self {
        self.code = Some(CodeableConcept::from_coding(Coding::loinc(code, display)));
        self
    }

    /// Set the patient reference.
    pub fn patient_reference(mut self, reference: impl Into<String>) -> Self {
        self.subject = Some(Reference::new(reference));
        self
    }

    /// Set the encounter reference.
    pub fn encounter_reference(mut self, reference: impl Into<String>) -> Self {
        self.encounter = Some(Reference::new(reference));
        self
    }

    /// Set the effective datetime (ISO 8601 string).
    pub fn effective_datetime(mut self, dt: impl Into<String>) -> Self {
        self.effective_date_time = Some(dt.into());
        self
    }

    /// Set a numeric value with unit.
    pub fn value_quantity(mut self, value: f64, unit: impl Into<String>) -> Self {
        self.value_quantity = Some(Quantity::new(value, unit));
        self
    }

    /// Build the Observation. Panics if required fields (status, code) are missing.
    pub fn build(self) -> Observation {
        // Vital-signs category
        let vital_signs_category = CodeableConcept::from_coding(Coding {
            system: Some("http://terminology.hl7.org/CodeSystem/observation-category".to_string()),
            code: Some("vital-signs".to_string()),
            display: Some("Vital Signs".to_string()),
        });

        Observation {
            resource_type: "Observation".to_string(),
            id: self.id,
            status: self.status.expect("ObservationBuilder: status is required"),
            category: Some(vec![vital_signs_category]),
            code: self.code.expect("ObservationBuilder: code is required"),
            subject: self.subject,
            encounter: self.encounter,
            effective_date_time: self.effective_date_time,
            value_quantity: self.value_quantity,
            reference_range: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_observation() {
        let obs = ObservationBuilder::new()
            .status(ObservationStatus::Final)
            .loinc_code("59408-5", "Oxygen saturation")
            .value_quantity(98.0, "%")
            .patient_reference("Patient/P001")
            .effective_datetime("2024-01-01T12:00:00+08:00")
            .build();

        assert_eq!(obs.resource_type, "Observation");
        assert_eq!(obs.value_quantity.unwrap().value, Some(98.0));
    }

    #[test]
    fn serializes_to_json() {
        let obs = ObservationBuilder::new()
            .status(ObservationStatus::Final)
            .loinc_code("8867-4", "Heart rate")
            .value_quantity(72.0, "/min")
            .build();

        let json = serde_json::to_string(&obs).unwrap();
        assert!(json.contains("\"resourceType\":\"Observation\""));
        assert!(json.contains("\"final\""));
    }
}
