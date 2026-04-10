//! FHIR R4 Patient resource stub.

use serde::{Deserialize, Serialize};

/// FHIR R4 Patient resource (minimal stub for reference linking).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patient {
    /// Always `"Patient"`.
    pub resource_type: String,

    /// Logical identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}
