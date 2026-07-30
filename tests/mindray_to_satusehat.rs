//! End-to-end pipeline test (T2.1): Mindray HL7 ORU^R01 -> hl7-mindray vitals
//! -> FHIR R4 Observation -> SATUSEHAT-profiled JSON, for three Mindray device
//! fixtures (ePM series, BeneVision N-series PDS, iPM 9800).
//!
//! The vital-sign-to-Observation mapping below is test-only glue: hl7-mindray
//! and fhir-r4 deliberately have no dependency on each other (see CLAUDE.md),
//! so this crate is where the two are wired together for an end-to-end check.

use fhir_r4::observation::{Observation, ObservationBuilder, ObservationStatus};
use hl7_mindray::{MindrayOru, VitalSign, codes};
use hl7_v2::Hl7Message;
use satusehat::observation::SatuSehatObservation;
use satusehat::{SatuSehatConfig, SatuSehatEnv};

fn test_config() -> SatuSehatConfig {
    SatuSehatConfig {
        env: SatuSehatEnv::Sandbox,
        client_id: "test-client".to_string(),
        client_secret: "test-secret".to_string(),
        organization_id: "org-hl7-rs-test".to_string(),
    }
}

/// Map a decoded vital sign to a FHIR Observation. Returns `None` for vital
/// signs that don't (yet) resolve to a stable code, matching the "skip
/// unmappable data rather than fail the batch" behavior a real pipeline needs.
fn observation_for_vital(vital: &VitalSign, patient_ref: &str) -> Option<Observation> {
    let (code, display, unit, value) = match vital {
        VitalSign::HeartRate(v) => (codes::HEART_RATE_LOINC, "Heart rate", "/min", *v),
        VitalSign::SpO2(v) => (codes::SPO2_LOINC, "Oxygen saturation", "%", *v),
        VitalSign::RespiratoryRate(v) => (codes::RESP_RATE_LOINC, "Respiratory rate", "/min", *v),
        VitalSign::Temperature(v) => (codes::TEMPERATURE_LOINC, "Body temperature", "Cel", *v),
        VitalSign::EtCO2(v) => (codes::ETCO2_MNDRY, "End-tidal CO2", "mmHg", *v),
        _ => return None,
    };

    let mut obs = ObservationBuilder::new()
        .status(ObservationStatus::Final)
        .loinc_code(code, display)
        .patient_reference(patient_ref)
        .value_quantity(value, unit)
        .build();

    // 99MNDRY codes aren't LOINC; correct the coding system for those.
    if code == codes::ETCO2_MNDRY
        && let Some(coding) = obs.code.coding.as_mut().and_then(|c| c.first_mut())
    {
        coding.system = Some("https://www.mindray.com/fhir/CodeSystem/99mndry".to_string());
    }

    Some(obs)
}

fn run_pipeline(raw: &str, patient_ref: &str) -> Vec<String> {
    let raw = raw.replace('\n', "\r").into_bytes();
    let message = Hl7Message::parse(&raw).expect("fixture parses as HL7 v2");
    let oru = MindrayOru::from_message(&message).expect("fixture is an ORU^R01 message");

    oru.vitals()
        .iter()
        .filter_map(|vital| observation_for_vital(vital, patient_ref))
        .map(|obs| {
            let satusehat_obs = SatuSehatObservation::from_observation(obs, &test_config());
            satusehat_obs
                .to_json()
                .expect("serializes to SATUSEHAT JSON")
        })
        .collect()
}

#[test]
fn epm_series_vitals_reach_satusehat_json() {
    let raw = include_str!("fixtures/mindray_epm_series.hl7");
    let jsons = run_pipeline(raw, "Patient/P10001");

    assert_eq!(jsons.len(), 4, "SpO2, HR, RR, and Temp should all map");
    for json in &jsons {
        assert!(json.contains("\"resourceType\": \"Observation\""));
        assert!(json.contains("Patient/P10001"));
        assert!(json.contains("org-hl7-rs-test"));
        assert!(json.contains("satusehat"));
    }
    assert!(jsons.iter().any(|j| j.contains("\"value\": 97.0")));
    assert!(jsons.iter().any(|j| j.contains("\"value\": 76.0")));
    assert!(jsons.iter().any(|j| j.contains("\"value\": 18.0")));
    assert!(jsons.iter().any(|j| j.contains("\"value\": 36.8")));
}

#[test]
fn benevision_n_series_pds_vitals_reach_satusehat_json() {
    let raw = include_str!("fixtures/mindray_benevision_n_series.hl7");
    let jsons = run_pipeline(raw, "Patient/P10002");

    assert_eq!(jsons.len(), 3, "HR, SpO2, and EtCO2 should all map");
    assert!(jsons.iter().any(|j| j.contains("\"value\": 88.0")));
    assert!(jsons.iter().any(|j| j.contains("\"value\": 99.0")));
    assert!(
        jsons
            .iter()
            .any(|j| j.contains("99mndry") && j.contains("\"value\": 36.0"))
    );
}

#[test]
fn ipm_9800_vitals_reach_satusehat_json() {
    let raw = include_str!("fixtures/mindray_ipm_9800.hl7");
    let jsons = run_pipeline(raw, "Patient/P10003");

    assert_eq!(jsons.len(), 2, "HR and Temp should both map");
    assert!(jsons.iter().any(|j| j.contains("\"value\": 70.0")));
    assert!(jsons.iter().any(|j| j.contains("\"value\": 37.1")));
}
