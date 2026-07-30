//! Encode a parsed ORU^R01 [`Hl7Message`] into a [`RecordBatch`] matching
//! [`oru_r01_schema`](crate::oru_r01_schema).

use std::fmt;
use std::sync::Arc;

use arrow::array::{
    ArrayRef, Float64Array, ListArray, RecordBatch, StringArray, StructArray,
    TimestampMicrosecondArray,
};
use arrow::buffer::{OffsetBuffer, ScalarBuffer};
use arrow::datatypes::DataType;
use hl7_v2::{Hl7Message, Segment};

use crate::datetime::parse_hl7_datetime;
use crate::schema::{list_field, obr_fields, obx_fields, oru_r01_schema};

/// Error returned by [`encode_oru_r01`] when the source message is missing
/// something the ORU^R01 schema requires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// The message has no `MSH` segment.
    MissingMsh,
    /// `MSH-10` (message control ID) is empty; the schema requires it.
    MissingMessageControlId,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::MissingMsh => write!(f, "message has no MSH segment"),
            EncodeError::MissingMessageControlId => {
                write!(f, "MSH-10 (message control ID) is required but empty")
            }
        }
    }
}

impl std::error::Error for EncodeError {}

struct ObxRow {
    set_id: Option<String>,
    value_type: String,
    observation_identifier_code: Option<String>,
    observation_identifier_text: Option<String>,
    value_numeric: Option<f64>,
    value_text: Option<String>,
    value_timestamp: Option<i64>,
    units: Option<String>,
    reference_range: Option<String>,
    abnormal_flags: Option<String>,
    observation_result_status: Option<String>,
}

struct ObrGroup {
    set_id: Option<String>,
    universal_service_id_code: Option<String>,
    universal_service_id_text: Option<String>,
    observation_datetime: Option<i64>,
    obx: Vec<ObxRow>,
}

// `Field::value()`/`component()` return `&str` elided to the temporary
// `Field`'s own lifetime, not to the message buffer — so the borrow must be
// converted to an owned `String` before that temporary drops, inside the
// same closure that produced it (see hl7-v2's own note on this tradeoff).
fn field_opt(seg: &Segment<'_>, index: usize) -> Option<String> {
    seg.field(index).and_then(|f| {
        let v = f.value();
        (!v.is_empty()).then(|| v.to_string())
    })
}

fn component_opt(seg: &Segment<'_>, index: usize, component: usize) -> Option<String> {
    seg.field(index).and_then(|f| {
        let v = f.component(component)?;
        (!v.is_empty()).then(|| v.to_string())
    })
}

fn datetime_opt(seg: &Segment<'_>, index: usize) -> Option<i64> {
    seg.field(index).and_then(|f| parse_hl7_datetime(f.value()))
}

fn build_obr_group(seg: &Segment<'_>) -> ObrGroup {
    ObrGroup {
        set_id: field_opt(seg, 1),
        universal_service_id_code: component_opt(seg, 4, 1),
        universal_service_id_text: component_opt(seg, 4, 2),
        observation_datetime: datetime_opt(seg, 7),
        obx: Vec::new(),
    }
}

fn build_obx_row(seg: &Segment<'_>) -> ObxRow {
    let value_type = field_opt(seg, 2).unwrap_or_default();
    let raw_value = field_opt(seg, 5);

    let (value_numeric, value_text, value_timestamp) = match value_type.as_str() {
        "NM" => (
            raw_value.as_deref().and_then(|v| v.parse::<f64>().ok()),
            None,
            None,
        ),
        "TS" => (
            None,
            None,
            raw_value.as_deref().and_then(parse_hl7_datetime),
        ),
        _ => (None, raw_value, None),
    };

    ObxRow {
        set_id: field_opt(seg, 1),
        value_type,
        observation_identifier_code: component_opt(seg, 3, 1),
        observation_identifier_text: component_opt(seg, 3, 2),
        value_numeric,
        value_text,
        value_timestamp,
        units: field_opt(seg, 6),
        reference_range: field_opt(seg, 7),
        abnormal_flags: field_opt(seg, 8),
        observation_result_status: field_opt(seg, 11),
    }
}

fn utf8_array<'a>(values: impl Iterator<Item = Option<&'a str>>) -> StringArray {
    StringArray::from_iter(values)
}

fn build_obx_struct_array(rows: &[&ObxRow]) -> StructArray {
    let set_id: Vec<Option<&str>> = rows.iter().map(|r| r.set_id.as_deref()).collect();
    let value_type: Vec<Option<&str>> = rows.iter().map(|r| Some(r.value_type.as_str())).collect();
    let obs_id_code: Vec<Option<&str>> = rows
        .iter()
        .map(|r| r.observation_identifier_code.as_deref())
        .collect();
    let obs_id_text: Vec<Option<&str>> = rows
        .iter()
        .map(|r| r.observation_identifier_text.as_deref())
        .collect();
    let value_numeric: Vec<Option<f64>> = rows.iter().map(|r| r.value_numeric).collect();
    let value_text: Vec<Option<&str>> = rows.iter().map(|r| r.value_text.as_deref()).collect();
    let value_timestamp: Vec<Option<i64>> = rows.iter().map(|r| r.value_timestamp).collect();
    let units: Vec<Option<&str>> = rows.iter().map(|r| r.units.as_deref()).collect();
    let reference_range: Vec<Option<&str>> =
        rows.iter().map(|r| r.reference_range.as_deref()).collect();
    let abnormal_flags: Vec<Option<&str>> =
        rows.iter().map(|r| r.abnormal_flags.as_deref()).collect();
    let result_status: Vec<Option<&str>> = rows
        .iter()
        .map(|r| r.observation_result_status.as_deref())
        .collect();

    let columns: Vec<ArrayRef> = vec![
        Arc::new(utf8_array(set_id.into_iter())),
        Arc::new(utf8_array(value_type.into_iter())),
        Arc::new(utf8_array(obs_id_code.into_iter())),
        Arc::new(utf8_array(obs_id_text.into_iter())),
        Arc::new(Float64Array::from(value_numeric)),
        Arc::new(utf8_array(value_text.into_iter())),
        Arc::new(TimestampMicrosecondArray::from(value_timestamp)),
        Arc::new(utf8_array(units.into_iter())),
        Arc::new(utf8_array(reference_range.into_iter())),
        Arc::new(utf8_array(abnormal_flags.into_iter())),
        Arc::new(utf8_array(result_status.into_iter())),
    ];
    StructArray::try_new(obx_fields(), columns, None).expect("obx columns match obx_fields()")
}

fn build_obr_list_array(groups: &[ObrGroup]) -> ArrayRef {
    // Flatten every group's OBX rows into one array, tracking each group's
    // sub-range so the nested ListArray's offsets can slice back into it.
    let mut obx_offsets: Vec<i32> = Vec::with_capacity(groups.len() + 1);
    obx_offsets.push(0);
    let mut flat_obx: Vec<&ObxRow> = Vec::new();
    for group in groups {
        flat_obx.extend(group.obx.iter());
        obx_offsets.push(flat_obx.len() as i32);
    }

    let obx_struct = build_obx_struct_array(&flat_obx);
    let obx_list = ListArray::new(
        list_field(DataType::Struct(obx_fields())),
        OffsetBuffer::new(ScalarBuffer::from(obx_offsets)),
        Arc::new(obx_struct),
        None,
    );

    let set_id: Vec<Option<&str>> = groups.iter().map(|g| g.set_id.as_deref()).collect();
    let service_code: Vec<Option<&str>> = groups
        .iter()
        .map(|g| g.universal_service_id_code.as_deref())
        .collect();
    let service_text: Vec<Option<&str>> = groups
        .iter()
        .map(|g| g.universal_service_id_text.as_deref())
        .collect();
    let obs_datetime: Vec<Option<i64>> = groups.iter().map(|g| g.observation_datetime).collect();

    let obr_columns: Vec<ArrayRef> = vec![
        Arc::new(utf8_array(set_id.into_iter())),
        Arc::new(utf8_array(service_code.into_iter())),
        Arc::new(utf8_array(service_text.into_iter())),
        Arc::new(TimestampMicrosecondArray::from(obs_datetime)),
        Arc::new(obx_list),
    ];
    let obr_struct = StructArray::try_new(obr_fields(), obr_columns, None)
        .expect("obr columns match obr_fields()");

    let obr_list = ListArray::new(
        list_field(DataType::Struct(obr_fields())),
        OffsetBuffer::new(ScalarBuffer::from(vec![0i32, groups.len() as i32])),
        Arc::new(obr_struct),
        None,
    );
    Arc::new(obr_list)
}

/// Encode a parsed ORU^R01 [`Hl7Message`] into a single-row Arrow
/// [`RecordBatch`] matching [`oru_r01_schema`](crate::oru_r01_schema).
///
/// Walks segments in original document order (via
/// [`Hl7Message::all_segments`]) so each `OBX` is attached to the `OBR`
/// group it followed in the source message. `OBX` segments encountered
/// before the first `OBR` are dropped — ORU^R01 without a preceding order
/// group is out of scope for the v0.1 schema.
pub fn encode_oru_r01(message: &Hl7Message<'_>) -> Result<RecordBatch, EncodeError> {
    let msh = message.msh().ok_or(EncodeError::MissingMsh)?;

    // MSH-1 is the field separator itself and isn't stored in
    // `Segment::field()`, so every other MSH-N sits one position back:
    // `field(N - 1)` == MSH-N.
    let sending_application = field_opt(msh, 2);
    let sending_facility = field_opt(msh, 3);
    let receiving_application = field_opt(msh, 4);
    let receiving_facility = field_opt(msh, 5);
    let message_datetime = datetime_opt(msh, 6);
    let message_control_id = field_opt(msh, 9).ok_or(EncodeError::MissingMessageControlId)?;

    let (patient_id, patient_name, date_of_birth, sex) = match message.segment("PID") {
        Some(pid) => (
            field_opt(pid, 3),
            field_opt(pid, 5),
            datetime_opt(pid, 7),
            field_opt(pid, 8),
        ),
        None => (None, None, None, None),
    };

    let mut groups: Vec<ObrGroup> = Vec::new();
    for segment in message.all_segments() {
        match segment.name {
            "OBR" => groups.push(build_obr_group(segment)),
            "OBX" => {
                if let Some(group) = groups.last_mut() {
                    group.obx.push(build_obx_row(segment));
                }
            }
            _ => {}
        }
    }

    let obr_array = build_obr_list_array(&groups);

    let schema = Arc::new(oru_r01_schema());
    let columns: Vec<ArrayRef> = vec![
        Arc::new(utf8_array(std::iter::once(sending_application.as_deref()))),
        Arc::new(utf8_array(std::iter::once(sending_facility.as_deref()))),
        Arc::new(utf8_array(std::iter::once(
            receiving_application.as_deref(),
        ))),
        Arc::new(utf8_array(std::iter::once(receiving_facility.as_deref()))),
        Arc::new(TimestampMicrosecondArray::from(vec![message_datetime])),
        Arc::new(utf8_array(std::iter::once(Some(
            message_control_id.as_str(),
        )))),
        Arc::new(utf8_array(std::iter::once(patient_id.as_deref()))),
        Arc::new(utf8_array(std::iter::once(patient_name.as_deref()))),
        Arc::new(TimestampMicrosecondArray::from(vec![date_of_birth])),
        Arc::new(utf8_array(std::iter::once(sex.as_deref()))),
        obr_array,
    ];

    Ok(RecordBatch::try_new(schema, columns).expect("encoded columns match oru_r01_schema()"))
}
