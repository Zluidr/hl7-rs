# hl7-arrow

*(planned, v0.1 — this document is the schema design deliverable for HT-T10.1; no code exists yet, see [`TODO.md`](../../TODO.md) HT-T10 for crate scaffolding)*

Apache Arrow RecordBatch emission for parsed HL7 v2 messages (`hl7-v2::Hl7Message`). One job: turn a typed HL7 v2 AST into columnar Arrow data. No opinion on transport (stdout, file, socket, Flight, shared memory — see root [`README.md`](../../README.md#cross-language-integration-via-apache-arrow-v01-planned)).

## Schema-version matrix

Each message-type schema is versioned independently so a Python/TypeScript/R/Julia consumer can detect a shape change without parsing field-by-field.

| Message type | Schema function | Schema version | Status |
|---|---|---|---|
| ORU^R01 | `oru_r01_schema()` | `1.0.0` | designed (this document) |
| ADT | `adt_schema()` | — | not yet designed |
| ORM | `orm_schema()` | — | not yet designed |
| MDM | `mdm_schema()` | — | not yet designed |

The version is not a data column — it is embedded once per batch as Arrow schema-level metadata (see [Schema-version metadata](#schema-version-metadata)), since it describes the batch's shape, not any single row.

## RecordBatch schema: ORU^R01

One row per ORU^R01 message. Repeating HL7 segments (`OBR`, `OBX`) become nested `List<Struct<...>>` columns rather than being flattened into separate RecordBatches — this keeps one message = one row, so row count is meaningful and a consumer can `explode()`/`flatten()` in whichever language they're in.

### Top-level fields (MSH / PID)

| Field | Arrow type | Nullable | HL7 source |
|---|---|---|---|
| `msh.sending_application` | `Utf8` | yes | MSH-3 |
| `msh.sending_facility` | `Utf8` | yes | MSH-4 |
| `msh.receiving_application` | `Utf8` | yes | MSH-5 |
| `msh.receiving_facility` | `Utf8` | yes | MSH-6 |
| `msh.message_datetime` | `Timestamp(Microsecond, None)` | yes | MSH-7 |
| `msh.message_control_id` | `Utf8` | no | MSH-10 |
| `pid.patient_id` | `Utf8` | yes | PID-3 |
| `pid.patient_name` | `Utf8` | yes | PID-5 |
| `pid.date_of_birth` | `Timestamp(Microsecond, None)` | yes | PID-7 |
| `pid.sex` | `Utf8` | yes | PID-8 |
| `obr` | `List<Struct<...>>` | yes (empty list if no OBR groups) | see below |

### Nested: `obr` (repeating `OBR` + child `OBX` groups)

Struct fields inside each `obr` list element:

| Field | Arrow type | Nullable | HL7 source |
|---|---|---|---|
| `obr.set_id` | `Utf8` | yes | OBR-1 |
| `obr.universal_service_id_code` | `Utf8` | yes | OBR-4.1 |
| `obr.universal_service_id_text` | `Utf8` | yes | OBR-4.2 |
| `obr.observation_datetime` | `Timestamp(Microsecond, None)` | yes | OBR-7 |
| `obr.obx` | `List<Struct<...>>` | yes (empty list if no OBX in this OBR group) | see below |

### Nested: `obr.obx` (repeating `OBX` result observations)

Struct fields inside each `obx` list element:

| Field | Arrow type | Nullable | HL7 source |
|---|---|---|---|
| `obx.set_id` | `Utf8` | yes | OBX-1 |
| `obx.value_type` | `Utf8` | no | OBX-2 (e.g. `"NM"`, `"ST"`, `"TS"` — drives which `value_*` column below is populated) |
| `obx.observation_identifier_code` | `Utf8` | yes | OBX-3.1 (e.g. Mindray `99MNDRY` vendor code, see `hl7-mindray::codes`) |
| `obx.observation_identifier_text` | `Utf8` | yes | OBX-3.2 |
| `obx.value_numeric` | `Float64` | yes — populated only when `value_type == "NM"` | OBX-5 |
| `obx.value_text` | `Utf8` | yes — populated for `"ST"`/`"TX"`/other string-typed values | OBX-5 |
| `obx.value_timestamp` | `Timestamp(Microsecond, None)` | yes — populated only when `value_type == "TS"` | OBX-5 |
| `obx.units` | `Utf8` | yes | OBX-6 |
| `obx.reference_range` | `Utf8` | yes | OBX-7 |
| `obx.abnormal_flags` | `Utf8` | yes | OBX-8 |
| `obx.observation_result_status` | `Utf8` | yes | OBX-11 (e.g. `"F"` final, `"P"` preliminary, `"C"` corrected) |

**Type fidelity rule:** `value_numeric`/`value_text`/`value_timestamp` are three separate nullable columns rather than one variant/union column, because Arrow's `UnionArray` is awkward to consume from `pyarrow`/`arrow-js` compared to three plain nullable columns gated by `value_type`. Exactly one of the three is non-null per row (enforced by `encode_oru_r01`, not by the schema itself — Arrow has no cross-column constraint mechanism).

## Nullability / repetition rules

- A segment that HL7 v2 allows to repeat (`OBR`, `OBX`) is a `List<Struct<...>>` column, never flattened into repeated top-level columns (`obx_1`, `obx_2`, ...) — list length is unbounded and varies per message.
- A segment that's optional but non-repeating (e.g. `PID`, `PV1`) has its fields as plain nullable top-level columns — no wrapping `Option<Struct>`, since Arrow struct columns are already nullable at the row level if a whole segment is absent. (PID is currently modeled as always-present top-level fields since ORU^R01 without a PID is out of scope for v0.1; revisit if a fixture message needs it.)
- Every leaf field is nullable unless the source HL7 field is required by the message-type's own DoD gate — currently only `msh.message_control_id` (MSH-10) and `obx.value_type` (OBX-2) are non-nullable, since those are structurally required for the batch to be usable at all (control-id for message identity, value_type to know which `value_*` column to read).

## Segment grouping (struct/list columns)

Grouping mirrors HL7 v2's own ORDER_OBSERVATION structure: `OBR` groups own their child `OBX` observations. This is expressed as nested Arrow types (`obr: List<Struct<..., obx: List<Struct<...>>>>`) rather than as separate flat RecordBatches joined by a foreign key — Arrow's native nested-type support makes this a single self-contained batch with no join step required downstream.

## Schema-version metadata

`oru_r01_schema()` returns an Arrow `Schema` whose `metadata()` map carries:

| Key | Value |
|---|---|
| `hl7_arrow.schema_version` | `"1.0.0"` |
| `hl7_arrow.message_type` | `"ORU^R01"` |

A consumer reads this from the IPC stream's schema message before processing any batch, so a schema-version bump (e.g. a field rename or type change) is detectable without inspecting row data. Per `TODO.md`'s T2.4, a schema-hash diff check will track breaking changes to this document independently of the crate's own SemVer.

## Neutral HL7 field naming

Columns use canonical HL7 v2 segment/field identifiers in `snake_case` (`msh.sending_application`, `obx.value_numeric`), not names tied to any specific downstream data model (SNOMED, LOINC, FHIR Observation, or Mindray's `99MNDRY` vendor codes). Consumers that want a domain-specific mapping (e.g. mapping `obx.observation_identifier_code` through `hl7-mindray::codes` to a `VitalSign` variant) do that in their own code, same posture as every other crate in this workspace.

## Non-goals for this document

- No Rust code, `Cargo.toml`, or crate scaffolding — that's HT-T10.
- No emission examples (`mllp_listener.rs` / `pyarrow_consumer.py`) — that's HT-T10.2.
- No schemas for ADT/ORM/MDM — only ORU^R01 is required by the HT-T10.1 DoD; the matrix above tracks them as not yet designed.
