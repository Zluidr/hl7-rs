# TODO — hl7-v2

Zero-dependency HL7 v2 message parser.

---

## Phase 0 — Foundation `[0.0.1]` ✅

- [x] `Hl7Message`, `Segment`, `Field`, `EncodingChars` types
- [x] `ParseError` with `Display` + `Error`
- [x] MSH encoding character extraction (reads from message, not hardcoded)
- [x] `message_type()`, `message_control_id()`, `version()` accessors
- [x] `segments(name)` iterator, `segment(name)` first-match
- [x] `raw_fields()` slice access
- [x] 6 unit tests + 1 doctest passing
- [x] Zero dependencies

---

## Phase 1 — Complete Parser `[0.1.0]`

### T1.1 — Field indexing correctness
- [ ] **Audit MSH field index semantics** — MSH is special: MSH-1 is `|` (separator), not in the fields array
  - [ ] Document clearly: for MSH, `field(N)` returns `fields[N-2]` (since MSH-1 = separator, MSH-2 = encoding chars)
  - [ ] For all other segments: `field(N)` returns `fields[N-1]`
  - [ ] Write test: MSH-3 = SendingApplication, MSH-4 = SendingFacility, confirm indices
- [ ] Fix or document `Field::value()` lifetime constraint
  - [ ] Current: `value()` returns `&str` borrowed from `Field` (local), not from the original input
  - [ ] Option A: change `field()` to return `Option<&str>` directly (simpler, less ergonomic)
  - [ ] Option B: keep `Field` but make it borrow from segment (lifetime threading)
  - [ ] Document chosen approach in module doc

### T1.2 — Repetition handling
- [ ] `Field::repetitions(&self) -> Vec<&str>` — split on `~` (repetition separator from encoding chars)
- [ ] `Field::repetition(index: usize) -> Option<&str>`
- [ ] Test: OBX-3 with two repetitions `59408-5~8867-4`

### T1.3 — Escape sequence handling
- [ ] `Field::unescape(&self) -> String` — process HL7 escape sequences
  - [ ] `\F\` → field separator
  - [ ] `\S\` → component separator
  - [ ] `\R\` → repetition separator
  - [ ] `\E\` → escape character
  - [ ] `\T\` → sub-component separator
  - [ ] `\Hxxx\` → highlighted text (strip or preserve, configurable)
- [ ] Test: escaped field separator in patient name

### T1.4 — Typed segment accessors
- [ ] `MshSegment` wrapper (newtype over `Segment`)
  - [ ] `sending_application() -> Option<&str>` — MSH-3
  - [ ] `sending_facility() -> Option<&str>` — MSH-4
  - [ ] `message_datetime() -> Option<&str>` — MSH-7
  - [ ] `message_type() -> Option<&str>` — MSH-9
  - [ ] `message_control_id() -> Option<&str>` — MSH-10
  - [ ] `processing_id() -> Option<&str>` — MSH-11
  - [ ] `version_id() -> Option<&str>` — MSH-12
- [ ] `ObxSegment` wrapper
  - [ ] `set_id() -> Option<&str>` — OBX-1
  - [ ] `value_type() -> Option<&str>` — OBX-2
  - [ ] `observation_identifier() -> Option<&str>` — OBX-3 (full)
  - [ ] `observation_code() -> Option<&str>` — OBX-3.1 (first component)
  - [ ] `observation_value() -> Option<&str>` — OBX-5
  - [ ] `units() -> Option<&str>` — OBX-6
  - [ ] `observation_status() -> Option<&str>` — OBX-11 (F=final, P=prelim, etc.)
- [ ] `PidSegment` wrapper
  - [ ] `patient_id() -> Option<&str>` — PID-3.1
  - [ ] `patient_name() -> Option<(&str, &str)>` — PID-5 (family, given)
  - [ ] `date_of_birth() -> Option<&str>` — PID-7

### T1.5 — Message type helpers
- [ ] `Hl7Message::is_oru_r01(&self) -> bool`
- [ ] `Hl7Message::is_adt(&self) -> bool` — any ADT event
- [ ] `Hl7Message::event_type(&self) -> Option<&str>` — MSH-9 second component (e.g. "R01", "A01")

### T1.6 — `\r\n` and encoding edge cases
- [ ] Test: segments separated by `\r\n` (Windows line endings) — currently `trim_end_matches('\n')`
- [ ] Test: trailing `\r` at end of message
- [ ] Test: empty segment names (skip gracefully)
- [ ] Test: MSH with non-standard encoding characters
- [ ] Test: message with only MSH (no other segments)

### T1.7 — Documentation
- [ ] Module doc: explain HL7 v2 structure (MSH, segments, fields, components)
- [ ] Explain the MSH-1/MSH-2 special handling clearly with example
- [ ] `Field::value()` — document lifetime constraint explicitly
- [ ] `Field::component()` — document 1-indexed behaviour
- [ ] Add `examples/parse_oru.rs` — parse a complete ORU^R01, print all OBX values

### T1.8 — Tests
- [ ] Test: PID segment — patient name extraction
- [ ] Test: OBX set with 5+ observations
- [ ] Test: message with non-default encoding characters
- [ ] Test: UTF-8 multibyte characters in patient name
- [ ] Test: MSH-7 datetime field extraction
- [ ] Test: `is_oru_r01()` true and false cases

---

## Phase 2 — Robustness `[0.2.0]`

### T2.1 — HL7 version matrix
- [ ] Document tested HL7 versions: 2.3, 2.3.1, 2.4, 2.5, 2.5.1, 2.6
- [ ] Identify version-specific segment differences (if any affect parsing)
- [ ] Add `version()` to return `Hl7Version` enum vs raw `&str`

### T2.2 — Builder (outbound messages)
- [ ] `Hl7MessageBuilder` — construct outbound HL7 v2 messages
  - [ ] `new(message_type: &str) -> Self`
  - [ ] `add_segment(name: &str, fields: Vec<&str>) -> Self`
  - [ ] `build() -> String` — serializes to HL7 wire format
- [ ] Test: build then parse round-trip

### T2.3 — Performance
- [ ] Profile `parse()` allocations — `Vec<Vec<&str>>` per segment
- [ ] Benchmark: 1000 ORU^R01 messages/second target
- [ ] Consider lazy parsing variant (only parse segments on demand)

---

## Phase 3 — Stable `[1.0.0]`

### T3.1 — API freeze
- [ ] Confirm `Field::value()` lifetime design is final
- [ ] Confirm typed segment API surface is stable
- [ ] `cargo semver-checks` passes

### T3.2 — Quality gates
- [ ] Zero `cargo doc` warnings
- [ ] Zero `cargo clippy -- -D warnings`
- [ ] Test coverage ≥ 80%
- [ ] `cargo audit` clean
- [ ] CHANGELOG complete

### T3.3 — Release
- [ ] Tag `hl7-v2-v1.0.0`
- [ ] Publish to crates.io
- [ ] GitHub Release
