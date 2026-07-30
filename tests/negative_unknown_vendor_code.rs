//! Negative test (T2.1): an unrecognized 99MNDRY vendor code must decode to
//! `VitalSign::Unknown` rather than erroring or panicking, and must be safely
//! skippable by downstream FHIR mapping instead of breaking the whole batch.

use hl7_mindray::{MindrayOru, VitalSign};
use hl7_v2::Hl7Message;

const RAW: &[u8] = b"MSH|^~\\&|BeneVisionN19|OR2|EMR||20240115110000||ORU^R01|MSG00104|P|2.3.1\r\
PID|1||P10004^^^MRN||Rahman^Dewi||19881130|F\r\
OBX|1|NM|8867-4^HR^LN||82|/min|60-100|N|||F\r\
OBX|2|NM|99MNDRY-NIBP-SYS^NIBP Sys^99MNDRY||118|mmHg|90-140|N|||F";

#[test]
fn unknown_vendor_code_becomes_unknown_variant_not_an_error() {
    let message = Hl7Message::parse(RAW).expect("fixture parses as HL7 v2");
    let oru = MindrayOru::from_message(&message).expect("fixture is an ORU^R01 message");

    assert_eq!(oru.vitals().len(), 2);
    assert_eq!(oru.vitals()[0], VitalSign::HeartRate(82.0));
    assert_eq!(
        oru.vitals()[1],
        VitalSign::Unknown {
            code: "99MNDRY-NIBP-SYS".to_string(),
            value: "118".to_string(),
            unit: Some("mmHg".to_string()),
        }
    );
}

#[test]
fn unknown_vendor_code_is_skippable_without_breaking_the_batch() {
    let message = Hl7Message::parse(RAW).expect("fixture parses as HL7 v2");
    let oru = MindrayOru::from_message(&message).expect("fixture is an ORU^R01 message");

    // Mirrors the filter_map skip-unmappable pattern used in
    // mindray_to_satusehat.rs: an Unknown vital must not panic or abort
    // processing of the rest of the batch.
    let mapped: Vec<&VitalSign> = oru
        .vitals()
        .iter()
        .filter(|v| !matches!(v, VitalSign::Unknown { .. }))
        .collect();

    assert_eq!(mapped, vec![&VitalSign::HeartRate(82.0)]);
}
