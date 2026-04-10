# TODO — satusehat

Indonesian SATUSEHAT national health platform — FHIR R4 profiles, API client, and data models.

Reference: [SATUSEHAT Developer Portal](https://satusehat.kemkes.go.id/platform) · Permenkes No. 24 Tahun 2022

---

## Phase 0 — Foundation `[0.0.1]` ✅

- [x] `SatuSehatEnv` — Sandbox / Production with base URLs
- [x] `SatuSehatConfig` — client_id, client_secret, organization_id
- [x] `AccessToken` — OAuth2 response struct
- [x] `SatuSehatError` — Json, MissingField, Http (feature-gated)
- [x] `SatuSehatObservation` — wraps FHIR R4 Observation with SATUSEHAT profile
- [x] `codes` module — SATUSEHAT system URIs and LOINC vital sign codes
- [x] 1 unit test + 1 doctest passing
- [x] Optional `client` feature (reqwest)

---

## Phase 1 — Core Integration `[0.1.0]`

### T1.1 — OAuth2 authentication
- [ ] `SatuSehatAuth` struct — manages token lifecycle
  - [ ] `async fn fetch_token(config: &SatuSehatConfig) -> Result<AccessToken, SatuSehatError>`
  - [ ] Uses `client_credentials` grant type
  - [ ] POST to `SatuSehatEnv::auth_url()` with `client_id` and `client_secret` as form body
  - [ ] Parse `AccessToken` from JSON response
- [ ] Token caching
  - [ ] `SatuSehatAuth::get_valid_token(&mut self) -> Result<&str, SatuSehatError>`
  - [ ] Cache token, auto-refresh when `expires_in - 60s` margin is reached
- [ ] Feature-gate entire `auth` module behind `client` feature
- [ ] Test: mock HTTP server returning valid token response
- [ ] Test: token refresh triggered when near expiry
- [ ] Document: SATUSEHAT token TTL is 900 seconds (15 min) as of 2024

### T1.2 — FHIR resource submission (`client` feature)
- [ ] `SatuSehatClient` struct
  - [ ] `async fn new(config: &SatuSehatConfig) -> Result<Self, SatuSehatError>` — fetches initial token
  - [ ] `async fn create_observation(&self, obs: &SatuSehatObservation) -> Result<String, SatuSehatError>` — returns resource ID
  - [ ] `async fn create_bundle(&self, bundle: &SatuSehatBundle) -> Result<BundleResponse, SatuSehatError>`
  - [ ] `async fn get_patient_by_nik(&self, nik: &str) -> Result<Option<Patient>, SatuSehatError>`
- [ ] Base URL from `SatuSehatEnv::fhir_base_url()`
- [ ] Request headers:
  - [ ] `Authorization: Bearer <token>`
  - [ ] `Content-Type: application/json`
- [ ] Retry logic: 3 attempts with exponential backoff on 5xx
- [ ] Rate limiting: respect `Retry-After` header

### T1.3 — SATUSEHAT Observation profile compliance
- [ ] Validate SATUSEHAT-required fields per [SATUSEHAT Observation profile](https://satusehat.kemkes.go.id)
  - [ ] `meta.profile` must include SATUSEHAT Observation StructureDefinition URL
  - [ ] `status` — only `final` or `amended` accepted
  - [ ] `subject.reference` — must be `Patient/<satusehat_patient_id>` (not local MRN)
  - [ ] `encounter.reference` — must be `Encounter/<satusehat_encounter_id>` if present
  - [ ] `performer` — organization reference required
  - [ ] `effectiveDateTime` — ISO 8601 with timezone required
- [ ] `SatuSehatObservation::validate(&self) -> Vec<ValidationIssue>` — check all required fields
- [ ] Test: missing `effectiveDateTime` → validation error
- [ ] Test: correct profile URL injected

### T1.4 — Patient identity resolution
- [ ] SATUSEHAT requires national Patient IDs (from SATUSEHAT, not local MRN)
  - [ ] `async fn resolve_patient_nik(nik: &str, name: &str, dob: &str) -> Result<String, SatuSehatError>` — returns SATUSEHAT patient ID
  - [ ] Uses SATUSEHAT Patient Demographics Query endpoint
- [ ] `PatientIdentity` struct: `{ satusehat_id: String, nik: String, name: String }`
- [ ] Cache resolved identities (in-memory, configurable TTL)
- [ ] Test: NIK lookup returns SATUSEHAT patient ID

### T1.5 — `SatuSehatBundle` (transaction bundle for batch)
- [ ] `SatuSehatBundle` wraps `fhir_r4::Bundle`
  - [ ] Auto-generates entry `fullUrl` in format `urn:uuid:<uuid>`
  - [ ] Sets `request.method = "POST"`, `request.url = "<ResourceType>"`
  - [ ] Injects SATUSEHAT profile into each resource's `meta.profile`
- [ ] `SatuSehatBundleBuilder`
  - [ ] `add_observation(obs: SatuSehatObservation) -> Self`
  - [ ] `build() -> SatuSehatBundle`
- [ ] Test: 5 observations → bundle with correct entry structure
- [ ] Test: `fullUrl` format validation

### T1.6 — Sandbox vs Production
- [ ] `SatuSehatEnv::Sandbox` — default for development
- [ ] Add `SatuSehatEnv::Staging` — `dto.kemkes.go.id/stg` (if endpoint exists)
- [ ] Document Sandbox limitations (test data only, no real patients)
- [ ] Add env configuration from environment variables
  - [ ] `SatuSehatConfig::from_env() -> Result<Self, SatuSehatError>`
  - [ ] Reads: `SATUSEHAT_CLIENT_ID`, `SATUSEHAT_CLIENT_SECRET`, `SATUSEHAT_ORG_ID`, `SATUSEHAT_ENV`

### T1.7 — `codes` module completion
- [ ] Verify all LOINC codes in `vital_sign_categories` against SATUSEHAT implementation guide
  - [ ] SpO2: 59408-5 ✓
  - [ ] Heart rate: 8867-4 ✓
  - [ ] Respiratory rate: 9279-1 ✓
  - [ ] Body temperature: 8310-5 ✓
  - [ ] NIBP systolic: 8480-6 ✓
  - [ ] NIBP diastolic: 8462-4 ✓
  - [ ] MAP: 8478-0 ✓
  - [ ] Add: NIBP mean 8478-0 (check — may differ)
  - [ ] Add: IBP systolic (invasive)
  - [ ] Add: EtCO2 LOINC code
- [ ] Add `icd10` module — common Indonesian ICD-10 codes for ICU (sepsis, pneumonia, etc.)
- [ ] Add `snomed` module — SNOMED CT codes used in SATUSEHAT profiles

### T1.8 — Documentation
- [ ] Module doc: explain SATUSEHAT platform, Permenkes No. 24/2022 mandate
- [ ] Document authentication flow with sequence diagram (ASCII)
- [ ] Document Sandbox registration process (link to Kemenkes portal)
- [ ] `SatuSehatClient` — document all methods with examples
- [ ] Add `examples/submit_observation.rs` — full flow: auth → build obs → submit
- [ ] Add `examples/resolve_patient.rs` — NIK → SATUSEHAT patient ID lookup

### T1.9 — Tests
- [ ] Test: `SatuSehatConfig::from_env()` with all env vars set
- [ ] Test: `SatuSehatConfig::from_env()` with missing var returns descriptive error
- [ ] Test: `SatuSehatObservation::validate()` all required field checks
- [ ] Test: bundle entry `fullUrl` is valid UUID URN format
- [ ] Integration test (sandbox only, skipped in CI unless credentials provided):
  - [ ] `#[ignore]` — run with `cargo test -- --ignored`
  - [ ] Full round-trip: auth → resolve patient → submit observation

---

## Phase 2 — Compliance & Production `[0.2.0]`

### T2.1 — Permenkes No. 24/2022 resource coverage
- [ ] Audit which FHIR resources are mandated for hospital integration
  - [ ] Encounter (rawat inap, rawat jalan)
  - [ ] Condition (diagnosis)
  - [ ] Procedure
  - [ ] MedicationRequest
  - [ ] AllergyIntolerance
  - [ ] Observation (vital signs — already covered)
- [ ] Add SATUSEHAT profile wrappers for each mandated resource
- [ ] Compliance checklist — track per-resource implementation status

### T2.2 — Error handling improvements
- [ ] Map SATUSEHAT API error codes to typed errors
  - [ ] 400: `SatuSehatError::InvalidResource { details: String }`
  - [ ] 401: `SatuSehatError::Unauthorized`
  - [ ] 403: `SatuSehatError::Forbidden`
  - [ ] 404: `SatuSehatError::NotFound { resource: String }`
  - [ ] 429: `SatuSehatError::RateLimited { retry_after: u64 }`
  - [ ] 5xx: `SatuSehatError::ServerError { status: u16 }`

### T2.3 — Webhook / subscription support (if SATUSEHAT supports)
- [ ] Research SATUSEHAT subscription model
- [ ] If supported: `SatuSehatSubscription` struct
- [ ] Webhook signature verification

---

## Phase 3 — Stable `[1.0.0]`

### T3.1 — Compliance verification
- [ ] Run against SATUSEHAT Sandbox conformance tests (if available)
- [ ] Verify against current SATUSEHAT FHIR ImplementationGuide version
- [ ] All mandatory fields per Permenkes No. 24/2022 covered

### T3.2 — Quality gates
- [ ] Zero `cargo doc` warnings
- [ ] Zero `cargo clippy -- -D warnings`
- [ ] Test coverage ≥ 70% (lower due to network-dependent code)
- [ ] `cargo audit` clean
- [ ] CHANGELOG complete

### T3.3 — Release
- [ ] Tag `satusehat-v1.0.0`
- [ ] Publish to crates.io (depends on `fhir-r4 ≥ 1.0.0`)
- [ ] GitHub Release
