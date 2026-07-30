//! ORU^R01 Arrow schema (HT-T10.1 design — see the crate README for the
//! full field-by-field rationale).

use std::collections::HashMap;
use std::sync::Arc;

use arrow::datatypes::{DataType, Field, FieldRef, Fields, Schema, TimeUnit};

const SCHEMA_VERSION: &str = "1.0.0";
const MESSAGE_TYPE: &str = "ORU^R01";

fn timestamp_micros() -> DataType {
    DataType::Timestamp(TimeUnit::Microsecond, None)
}

/// Wrap `item_type` in a nullable `List` field named `"item"`, matching the
/// convention arrow-rs itself uses for list child fields. Shared by
/// [`oru_r01_schema`] and `encode_oru_r01` so the schema and the encoded
/// `RecordBatch` are always structurally identical.
pub(crate) fn list_field(item_type: DataType) -> FieldRef {
    Arc::new(Field::new("item", item_type, true))
}

/// Field layout of one `OBX` observation (nested inside an `OBR` group).
pub(crate) fn obx_fields() -> Fields {
    Fields::from(vec![
        Field::new("set_id", DataType::Utf8, true),
        Field::new("value_type", DataType::Utf8, false),
        Field::new("observation_identifier_code", DataType::Utf8, true),
        Field::new("observation_identifier_text", DataType::Utf8, true),
        Field::new("value_numeric", DataType::Float64, true),
        Field::new("value_text", DataType::Utf8, true),
        Field::new("value_timestamp", timestamp_micros(), true),
        Field::new("units", DataType::Utf8, true),
        Field::new("reference_range", DataType::Utf8, true),
        Field::new("abnormal_flags", DataType::Utf8, true),
        Field::new("observation_result_status", DataType::Utf8, true),
    ])
}

/// Field layout of one `OBR` order/result group, including its nested `OBX`
/// observations.
pub(crate) fn obr_fields() -> Fields {
    Fields::from(vec![
        Field::new("set_id", DataType::Utf8, true),
        Field::new("universal_service_id_code", DataType::Utf8, true),
        Field::new("universal_service_id_text", DataType::Utf8, true),
        Field::new("observation_datetime", timestamp_micros(), true),
        Field::new(
            "obx",
            DataType::List(list_field(DataType::Struct(obx_fields()))),
            true,
        ),
    ])
}

/// Arrow schema for HL7 v2 ORU^R01 messages.
///
/// Mirrors HL7's ORDER_OBSERVATION grouping: `obr` is a `List<Struct<...>>`
/// whose `obx` field is itself a `List<Struct<...>>` of that order's
/// observations. See the crate README for the full field/type/nullability
/// table this implements.
pub fn oru_r01_schema() -> Schema {
    let fields = vec![
        Field::new("msh.sending_application", DataType::Utf8, true),
        Field::new("msh.sending_facility", DataType::Utf8, true),
        Field::new("msh.receiving_application", DataType::Utf8, true),
        Field::new("msh.receiving_facility", DataType::Utf8, true),
        Field::new("msh.message_datetime", timestamp_micros(), true),
        Field::new("msh.message_control_id", DataType::Utf8, false),
        Field::new("pid.patient_id", DataType::Utf8, true),
        Field::new("pid.patient_name", DataType::Utf8, true),
        Field::new("pid.date_of_birth", timestamp_micros(), true),
        Field::new("pid.sex", DataType::Utf8, true),
        Field::new(
            "obr",
            DataType::List(list_field(DataType::Struct(obr_fields()))),
            true,
        ),
    ];

    let metadata = HashMap::from([
        (
            "hl7_arrow.schema_version".to_string(),
            SCHEMA_VERSION.to_string(),
        ),
        (
            "hl7_arrow.message_type".to_string(),
            MESSAGE_TYPE.to_string(),
        ),
    ]);

    Schema::new(fields).with_metadata(metadata)
}
