# TODO — hl7-mindray

Mindray patient monitor HL7 field mappings — 99MNDRY private code space and vital sign extraction.

---

## Phase 0 — Foundation `[0.0.1]` ✅

- [x] `VitalSign` enum — HR, SpO2, RR, NIBP, Temp, EtCO2, IBP, Unknown
- [x] `MindrayOru` — parse from `Hl7Message`, extract typed vitals
- [x] `codes` module — LOINC and 99MNDRY constants
- [x] `MindrayError` — NotOru, MissingField, InvalidNumeric
- [x] 3 unit tests + 1 doctest passing
- [x] Depends on `hl7-v2` (workspace)

---

## Phase 1 — Complete Code Coverage `[0.1.0]`

### T1.1 — NIBP assembly
- [ ] **NIBP comes as 3 separate OBX messages** (sys, dia, mean) in Mindray PDS
  - [ ] `MindrayOru` must assemble them into one `VitalSign::Nibp`
  - [ ] Strategy: collect all OBX, post-process to merge NIBP components
  - [ ] `nibp() -> Option<VitalSign>` — returns assembled NIBP or None
- [ ] Map real 99MNDRY NIBP codes from Mindray PDS Programmer's Guide
  - [ ] Confirm actual code strings (currently using `"99MNDRY-NIBP-SYS"` as placeholder)
  - [ ] Replace placeholders with real Mindray OBX-3 identifier values
  - [ ] Add source reference comment linking to PDS guide section
- [ ] Test: 3-OBX NIBP message → single `VitalSign::Nibp { systolic, diastolic, mean }`
- [ ] Test: 2-OBX NIBP (no mean) → `mean: None`

### T1.2 — IBP channels
- [ ] Map IBP channel 1–4 codes from PDS guide
  - [ ] IBP channel naming is configurable on device (ART, CVP, PA, etc.)
  - [ ] Use channel number (1–4), not label, as the key
- [ ] `ibp(channel: u8) -> Option<VitalSign>` — assembled IBP for given channel
- [ ] Assemble IBP sys/dia/mean from separate OBX (same pattern as NIBP)
- [ ] Test: IBP channel 1 assembly
- [ ] Test: two IBP channels in same message

### T1.3 — EtCO2 mapping
- [ ] Confirm real 99MNDRY EtCO2 code from PDS guide
  - [ ] Replace `"99MNDRY-ETCO2"` placeholder with actual code
- [ ] Add `etco2() -> Option<f64>` convenience accessor
- [ ] Test: EtCO2 extraction

### T1.4 — Device-specific quirks
- [ ] **BeneVision N-series (PDS protocol)**
  - [ ] Confirm message interval behaviour (configurable: 1s, 5s, etc.)
  - [ ] `MindrayDevice::BeneVisionN` variant for device-type context
  - [ ] N-series sends aperiodic params (NIBP) at last-measured value — document this
- [ ] **ePM series**
  - [ ] ePM uses standard HL7 output (less 99MNDRY, more standard LOINC)
  - [ ] Confirm DIAP protocol differences vs PDS
  - [ ] `MindrayDevice::EpmSeries` variant
- [ ] **iPM 9800**
  - [ ] Confirm HL7 version and field set
  - [ ] `MindrayDevice::Ipm9800` variant
- [ ] `MindrayOru::device_hint() -> Option<MindrayDevice>` — infer from MSH sending application

### T1.5 — `VitalSign` improvements
- [ ] `VitalSign::is_critical(&self) -> bool` — basic alarm-range check (configurable thresholds)
- [ ] `VitalSign::unit(&self) -> &str` — canonical unit string for each variant
- [ ] `VitalSign::loinc_code(&self) -> Option<&str>` — LOINC code for typed variants
- [ ] `impl Display for VitalSign` — human-readable format ("SpO2: 98%")

### T1.6 — Documentation
- [ ] Module doc: explain Mindray HL7 output architecture (PDS protocol, MLLP)
  - [ ] Explain 99MNDRY private code space with examples
  - [ ] Explain NIBP/IBP multi-OBX assembly requirement
- [ ] `codes` module: add reference to Mindray PDS Programmer's Guide per constant
- [ ] `MindrayOru`: document that `from_message` is non-destructive (can call multiple times)
- [ ] Add `examples/extract_vitals.rs` — parse a complete Mindray ORU, print all vitals

### T1.7 — Tests
- [ ] Test: message with all 5 standard vitals (HR, SpO2, RR, Temp, NIBP)
- [ ] Test: message with unknown 99MNDRY code → `VitalSign::Unknown` preserved
- [ ] Test: OBX with empty value → skipped (not panicked)
- [ ] Test: non-numeric OBX-5 for a numeric code → `VitalSign::Unknown`
- [ ] Test: `VitalSign::unit()` for each typed variant
- [ ] Test: `Display` formatting for each variant

---

## Phase 2 — Extended Device Support `[0.2.0]`

### T2.1 — Additional Mindray devices
- [ ] Research: A-series Anesthesia System HL7 output
  - [ ] Maps ventilator parameters (TV, RR, PEEP, FiO2, etc.)
  - [ ] Add `VitalSign::Ventilator { param: VentParam, value: f64, unit: String }`
- [ ] Research: VS-900 vital signs monitor
- [ ] Add device compatibility matrix to README

### T2.2 — Alarm state
- [ ] OBX-8 contains abnormal flags (H=high, L=low, HH=critical high, LL=critical low)
  - [ ] `AlarmState` enum: Normal, High, Low, CriticalHigh, CriticalLow
  - [ ] `VitalSignWithAlarm { vital: VitalSign, alarm: AlarmState }`
  - [ ] `MindrayOru::vitals_with_alarms() -> &[VitalSignWithAlarm]`

### T2.3 — Timestamp extraction
- [ ] OBR-7 contains observation datetime
  - [ ] `MindrayOru::observation_time() -> Option<&str>` — raw HL7 DTM
  - [ ] With optional `chrono` feature: parse to `DateTime<Utc>`

---

## Phase 3 — Stable `[1.0.0]`

### T3.1 — Code accuracy
- [ ] Cross-reference all 99MNDRY codes against Mindray's latest PDS guide
- [ ] Get confirmation from Mindray Indonesia on field mapping accuracy
- [ ] Note: Mindray may update codes across firmware versions — document version matrix

### T3.2 — Quality gates
- [ ] Zero `cargo doc` warnings
- [ ] Zero `cargo clippy -- -D warnings`
- [ ] Test coverage ≥ 75%
- [ ] CHANGELOG complete

### T3.3 — Release
- [ ] Tag `hl7-mindray-v1.0.0`
- [ ] Publish to crates.io (depends on `hl7-v2 ≥ 1.0.0`)
- [ ] GitHub Release
