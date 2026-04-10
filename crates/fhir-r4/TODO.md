# TODO — fhir-r4

FHIR R4 resource types and builders — focused on device integration pipelines.

---

## Phase 0 — Foundation `[0.0.1]` ✅

- [x] `Observation` resource with `ObservationBuilder`
- [x] `ObservationStatus` enum (all R4 status codes)
- [x] `Patient` stub
- [x] `types` module: `Reference`, `CodeableConcept`, `Coding`, `Quantity`
- [x] `Coding::loinc()`, `Coding::snomed()` constructors
- [x] `serde` JSON serialization with `camelCase` field names
- [x] `satusehat` feature flag (stub)
- [x] 2 unit tests + 1 doctest passing

---

## Phase 1 — Core Resources `[0.1.0]`

### T1.1 — `Observation` completeness
- [ ] Add `Observation::components` — `Vec<ObservationComponent>` for multi-value obs (e.g. NIBP sys+dia)
  - [ ] `ObservationComponent { code: CodeableConcept, value_quantity: Option<Quantity> }`
  - [ ] `ObservationBuilder::add_component(code, value, unit)`
- [ ] Add `Observation::note` — `Vec<Annotation>` for free-text notes
- [ ] Add `Observation::device` — `Option<Reference>` — link to device resource
- [ ] Add `Observation::based_on` — `Option<Vec<Reference>>` — ServiceRequest ref
- [ ] `ObservationBuilder` — validate required fields in `build()` return `Result` instead of panic
  - [ ] `build() -> Result<Observation, BuildError>` 
  - [ ] `build_unchecked() -> Observation` for performance-critical paths
- [ ] Test: NIBP observation with `components` (sys + dia)
- [ ] Test: `build()` with missing status returns `Err`

### T1.2 — `Encounter` resource
- [ ] `Encounter` struct
  - [ ] `resource_type: String` — always `"Encounter"`
  - [ ] `id: Option<String>`
  - [ ] `status: EncounterStatus` — planned, arrived, in-progress, finished, cancelled
  - [ ] `class: Coding` — inpatient (`IMP`), ambulatory (`AMB`), emergency (`EMER`)
  - [ ] `subject: Option<Reference>` — patient
  - [ ] `period: Option<Period>` — `{ start: Option<String>, end: Option<String> }`
  - [ ] `service_provider: Option<Reference>` — organization
- [ ] `EncounterBuilder`
  - [ ] `new() -> Self`
  - [ ] `inpatient() -> Self` — preset class
  - [ ] `subject(reference: &str) -> Self`
  - [ ] `build() -> Result<Encounter, BuildError>`
- [ ] Test: inpatient encounter serializes correctly
- [ ] Test: `"resourceType": "Encounter"` present in JSON

### T1.3 — `Patient` resource (upgrade from stub)
- [ ] Expand `Patient` struct
  - [ ] `id: Option<String>`
  - [ ] `identifier: Option<Vec<Identifier>>` — NIK, MRN, etc.
  - [ ] `name: Option<Vec<HumanName>>` — `{ family: Option<String>, given: Option<Vec<String>> }`
  - [ ] `gender: Option<Gender>` — male, female, other, unknown
  - [ ] `birth_date: Option<String>` — YYYY-MM-DD
  - [ ] `active: Option<bool>`
- [ ] `Identifier` type: `{ system: Option<String>, value: Option<String> }`
- [ ] Test: Indonesian NIK identifier format
- [ ] Test: `Patient` round-trip JSON

### T1.4 — `Bundle` resource
- [ ] `Bundle` struct — for batch submission to SATUSEHAT
  - [ ] `resource_type: String` — always `"Bundle"`
  - [ ] `id: Option<String>`
  - [ ] `type_: BundleType` — transaction, batch, collection (note: `type` is reserved)
  - [ ] `entry: Vec<BundleEntry>`
- [ ] `BundleEntry` struct
  - [ ] `resource: BundleResource` — enum wrapping Observation/Patient/Encounter
  - [ ] `request: Option<BundleRequest>` — for transaction bundles
- [ ] `BundleRequest`: `{ method: String, url: String }`
- [ ] `BundleBuilder`
  - [ ] `new(type_: BundleType) -> Self`
  - [ ] `add_observation(obs: Observation) -> Self`
  - [ ] `add_patient(pat: Patient) -> Self`
  - [ ] `build() -> Bundle`
- [ ] Test: transaction bundle with 1 Patient + 5 Observations

### T1.5 — `Organization` resource
- [ ] `Organization` struct
  - [ ] `id: Option<String>`
  - [ ] `identifier: Option<Vec<Identifier>>`
  - [ ] `name: Option<String>`
  - [ ] `type_: Option<Vec<CodeableConcept>>`
- [ ] Test: organization with SATUSEHAT identifier

### T1.6 — Shared types completion
- [ ] `Period` type: `{ start: Option<String>, end: Option<String> }`
- [ ] `HumanName` type: `{ family: Option<String>, given: Option<Vec<String>>, use_: Option<String> }`
- [ ] `Annotation` type: `{ text: String, time: Option<String> }`
- [ ] `Identifier` type (already needed above): `{ system: Option<String>, value: Option<String>, use_: Option<String> }`

### T1.7 — Documentation
- [ ] Module doc: explain FHIR R4 resource model (brief)
- [ ] `ObservationBuilder` — document every method
- [ ] `types` module — document FHIR type usage for each struct
- [ ] Add `examples/build_observation.rs` — build and serialize a vital sign Observation
- [ ] Add `examples/build_bundle.rs` — assemble a transaction bundle

### T1.8 — Tests
- [ ] Test: `Coding::loinc` and `Coding::snomed` produce correct `system` URIs
- [ ] Test: `Quantity::new` produces correct UCUM system URI
- [ ] Test: `Observation` JSON output matches FHIR R4 spec shape (spot-check keys)
- [ ] Test: `ObservationStatus` serializes to kebab-case (e.g. `"entered-in-error"`)

---

## Phase 2 — Validation & Profiles `[0.2.0]`

### T2.1 — FHIR validation (basic)
- [ ] `Observation::validate(&self) -> Vec<ValidationIssue>` — check required fields
  - [ ] `status` must be present
  - [ ] `code` must be present
  - [ ] `subject` reference format: must start with `Patient/`
- [ ] `ValidationIssue { severity: Severity, path: String, message: String }`

### T2.2 — Vital signs profile
- [ ] FHIR vital signs profile requires specific `category` coding
  - [ ] Validate that vital-signs observations have `http://terminology.hl7.org/CodeSystem/observation-category` category
  - [ ] Provide `ObservationBuilder::vital_sign()` preset that sets this automatically (currently does this — document it)

### T2.3 — `serde` improvements
- [ ] `#[serde(skip_serializing_if = "Option::is_none")]` — verify on all optional fields
- [ ] `BundleType` serializes as `"transaction"` not `"Transaction"` — verify
- [ ] Verify `valueQuantity` camelCase (not `value_quantity`)

---

## Phase 3 — Stable `[1.0.0]`

### T3.1 — API freeze
- [ ] `ObservationBuilder::build()` returns `Result` — final API decision
- [ ] All five core resources complete: Observation, Patient, Encounter, Bundle, Organization
- [ ] `cargo semver-checks` passes

### T3.2 — Quality gates
- [ ] Zero `cargo doc` warnings
- [ ] Zero `cargo clippy -- -D warnings`
- [ ] Test coverage ≥ 75%
- [ ] `cargo audit` clean
- [ ] CHANGELOG complete

### T3.3 — Release
- [ ] Tag `fhir-r4-v1.0.0`
- [ ] Publish to crates.io
- [ ] GitHub Release
