//! Rust-only Arrow round-trip test for the ORU^R01 schema (HT-T10.2).
//!
//! Encodes a fixture HL7 v2 ORU^R01 message to an Arrow `RecordBatch`,
//! writes it through an Arrow IPC stream, reads it back, and asserts the
//! deserialized field values match the source message. This is the
//! Rust-side half of the round-trip kill-criterion; the Python/pyarrow half
//! lives in `examples/pyarrow_consumer.py` and is not part of `cargo test`.

use arrow::array::{
    Array, Float64Array, ListArray, StringArray, StructArray, TimestampMicrosecondArray,
};
use arrow::ipc::reader::StreamReader;
use arrow::ipc::writer::StreamWriter;
use hl7_arrow::{EncodeError, encode_oru_r01};
use hl7_v2::Hl7Message;

const FIXTURE: &str = include_str!("fixtures/oru_r01_sample.hl7");

fn fixture_bytes() -> Vec<u8> {
    FIXTURE.replace('\n', "\r").into_bytes()
}

#[test]
fn oru_r01_round_trips_through_arrow_ipc() {
    let raw = fixture_bytes();
    let message = Hl7Message::parse(&raw).expect("fixture parses");
    let batch = encode_oru_r01(&message).expect("fixture encodes");

    // Write to an IPC stream, then read it back — this exercises the exact
    // path `mllp_listener.rs` uses to write to stdout.
    let mut ipc_bytes = Vec::new();
    {
        let mut writer = StreamWriter::try_new(&mut ipc_bytes, &batch.schema()).unwrap();
        writer.write(&batch).unwrap();
        writer.finish().unwrap();
    }

    let mut reader = StreamReader::try_new(ipc_bytes.as_slice(), None).unwrap();
    let read_batch = reader.next().unwrap().unwrap();
    assert!(reader.next().is_none(), "exactly one RecordBatch expected");

    assert_eq!(read_batch.num_rows(), 1);

    let schema = read_batch.schema();
    assert_eq!(
        schema
            .metadata()
            .get("hl7_arrow.schema_version")
            .map(String::as_str),
        Some("1.0.0")
    );
    assert_eq!(
        schema
            .metadata()
            .get("hl7_arrow.message_type")
            .map(String::as_str),
        Some("ORU^R01")
    );

    let col = |name: &str| read_batch.column(read_batch.schema().index_of(name).unwrap());

    let sending_application = col("msh.sending_application")
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(sending_application.value(0), "SendApp");

    let message_control_id = col("msh.message_control_id")
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(message_control_id.value(0), "MSG00001");

    let patient_id = col("pid.patient_id")
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(patient_id.value(0), "P12345^^^MRN");

    let sex = col("pid.sex")
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(sex.value(0), "F");

    // obr: List<Struct<...>> with 2 entries (two OBR groups in the fixture)
    let obr_list = col("obr").as_any().downcast_ref::<ListArray>().unwrap();
    let obr_struct = obr_list.value(0);
    let obr_struct = obr_struct.as_any().downcast_ref::<StructArray>().unwrap();
    assert_eq!(obr_struct.len(), 2);

    let obr_set_id = obr_struct
        .column_by_name("set_id")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(obr_set_id.value(0), "1");
    assert_eq!(obr_set_id.value(1), "2");

    let obr_service_code = obr_struct
        .column_by_name("universal_service_id_code")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(obr_service_code.value(0), "99MNDRY");
    assert_eq!(obr_service_code.value(1), "99MNDRY");

    let obr_service_text = obr_struct
        .column_by_name("universal_service_id_text")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(obr_service_text.value(0), "SpotCheckVitals");
    assert_eq!(obr_service_text.value(1), "ClinicalNote");

    // obr.obx: nested List<Struct<...>> — first OBR group has 2 NM observations
    let obx_list = obr_struct
        .column_by_name("obx")
        .unwrap()
        .as_any()
        .downcast_ref::<ListArray>()
        .unwrap();

    let first_group_obx = obx_list.value(0);
    let first_group_obx = first_group_obx
        .as_any()
        .downcast_ref::<StructArray>()
        .unwrap();
    assert_eq!(first_group_obx.len(), 2);

    let value_type = first_group_obx
        .column_by_name("value_type")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(value_type.value(0), "NM");
    assert_eq!(value_type.value(1), "NM");

    let value_numeric = first_group_obx
        .column_by_name("value_numeric")
        .unwrap()
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();
    assert_eq!(value_numeric.value(0), 97.0);
    assert_eq!(value_numeric.value(1), 72.0);

    let value_text = first_group_obx
        .column_by_name("value_text")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert!(value_text.is_null(0));
    assert!(value_text.is_null(1));

    let units = first_group_obx
        .column_by_name("units")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(units.value(0), "%");
    assert_eq!(units.value(1), "/min");

    let result_status = first_group_obx
        .column_by_name("observation_result_status")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(result_status.value(0), "F");

    // second OBR group: TS + ST observations exercise the other two value_* columns
    let second_group_obx = obx_list.value(1);
    let second_group_obx = second_group_obx
        .as_any()
        .downcast_ref::<StructArray>()
        .unwrap();
    assert_eq!(second_group_obx.len(), 2);

    let value_type_2 = second_group_obx
        .column_by_name("value_type")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(value_type_2.value(0), "TS");
    assert_eq!(value_type_2.value(1), "ST");

    let value_timestamp = second_group_obx
        .column_by_name("value_timestamp")
        .unwrap()
        .as_any()
        .downcast_ref::<TimestampMicrosecondArray>()
        .unwrap();
    assert!(!value_timestamp.is_null(0));
    assert!(value_timestamp.is_null(1));

    let value_text_2 = second_group_obx
        .column_by_name("value_text")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert!(value_text_2.is_null(0));
    assert_eq!(value_text_2.value(1), "Patient stable, no acute distress");

    // exactly one of value_numeric/value_text/value_timestamp populated per OBX row
    let value_numeric_2 = second_group_obx
        .column_by_name("value_numeric")
        .unwrap()
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();
    assert!(value_numeric_2.is_null(0));
    assert!(value_numeric_2.is_null(1));
}

#[test]
fn rejects_message_type_other_than_oru_r01() {
    let text = String::from_utf8(fixture_bytes())
        .unwrap()
        .replace("ORU^R01", "ADT^A01");
    let raw = text.into_bytes();
    let message = Hl7Message::parse(&raw).expect("fixture parses");
    assert_eq!(
        encode_oru_r01(&message),
        Err(EncodeError::UnsupportedMessageType("ADT^A01".to_string()))
    );
}

#[test]
fn treats_explicit_null_component_as_absent() {
    let text = String::from_utf8(fixture_bytes())
        .unwrap()
        .replace("99MNDRY^SpotCheckVitals", "\"\"^SpotCheckVitals");
    let raw = text.into_bytes();
    let message = Hl7Message::parse(&raw).expect("fixture parses");
    let batch = encode_oru_r01(&message).expect("fixture encodes");

    let obr_list = batch
        .column_by_name("obr")
        .unwrap()
        .as_any()
        .downcast_ref::<ListArray>()
        .unwrap();
    let obr_struct = obr_list
        .value(0)
        .as_any()
        .downcast_ref::<StructArray>()
        .unwrap()
        .clone();
    let service_code = obr_struct
        .column_by_name("universal_service_id_code")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert!(service_code.is_null(0));
}
