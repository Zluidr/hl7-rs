//! SATUSEHAT-profile Observation wrapper.

use crate::{SatuSehatConfig, SatuSehatError};
use fhir_r4::observation::Observation;
use serde_json::Value;

/// An Observation wrapped with SATUSEHAT profile requirements.
///
/// SATUSEHAT requires specific meta.profile, identifier patterns,
/// and organization references not present in base FHIR R4.
pub struct SatuSehatObservation {
    inner: Observation,
    organization_id: String,
}

impl SatuSehatObservation {
    /// Wrap a base FHIR R4 Observation with SATUSEHAT profile extensions.
    pub fn from_observation(obs: Observation, config: &SatuSehatConfig) -> Self {
        Self {
            inner: obs,
            organization_id: config.organization_id.clone(),
        }
    }

    /// Serialize to SATUSEHAT-compliant FHIR R4 JSON.
    ///
    /// Adds required SATUSEHAT profile metadata to the base Observation.
    pub fn to_json(&self) -> Result<String, SatuSehatError> {
        let mut value = serde_json::to_value(&self.inner)?;

        // Inject SATUSEHAT profile URI into meta
        if let Value::Object(ref mut map) = value {
            map.insert(
                "meta".to_string(),
                serde_json::json!({
                    "profile": [
                        "https://api-satusehat.kemkes.go.id/fhir/StructureDefinition/Observation-vital-signs"
                    ]
                }),
            );

            // Inject performer (organization reference) if not already set
            if !map.contains_key("performer") {
                map.insert(
                    "performer".to_string(),
                    serde_json::json!([{
                        "reference": format!("Organization/{}", self.organization_id)
                    }]),
                );
            }
        }

        serde_json::to_string_pretty(&value).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SatuSehatConfig, SatuSehatEnv};
    use fhir_r4::observation::{ObservationBuilder, ObservationStatus};

    #[test]
    fn adds_satusehat_profile() {
        let config = SatuSehatConfig {
            env: SatuSehatEnv::Sandbox,
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            organization_id: "org-001".to_string(),
        };

        let obs = ObservationBuilder::new()
            .status(ObservationStatus::Final)
            .loinc_code("59408-5", "Oxygen saturation")
            .value_quantity(98.0, "%")
            .build();

        let ss = SatuSehatObservation::from_observation(obs, &config);
        let json = ss.to_json().unwrap();

        assert!(json.contains("satusehat"));
        assert!(json.contains("org-001"));
    }
}
