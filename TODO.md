# TODO — hl7-rs workspace

Tracking all cross-crate work. Crate-specific tasks live in each crate's own `TODO.md`.

Version targets follow [SemVer](https://semver.org). Phase gates map to crate versions.

<!-- v0.1 (Apr 19, 2026): New "v0.1 Scope — hl7-arrow crate" section added below the intro. Prior Phases 0, 1, 2, 3 preserved verbatim with targeted inline annotations where the new crate intersects existing tasks. -->

---

## v0.1 Scope — `hl7-arrow` crate

> **Goal:** Ship a sibling crate that emits parsed HL7 v2 messages as Apache Arrow RecordBatches, so Rust-based HL7 ingestion can feed any Arrow-capable downstream (Python, TypeScript, R, Julia) without custom FFI or subprocess JSON round-tripping.
>
> **License impact:** None. All work remains Apache-2.0. `arrow-rs` (Apache-2.0) is compatible with `deny.toml` allow-list (T0.3). Confirm transitive deps on introduction (HT-T10.3).
>
> **SemVer impact:** New sibling crate `hl7-arrow` begins at its own `0.1.0`. Existing crates (`hl7-mllp`, `hl7-v2`, `hl7-mindray`, `fhir-r4`, `satusehat`) are **not affected** — `hl7-arrow` consumes `hl7-v2` as a read-only dependency. No breaking changes to existing public APIs.
>
> **Architectural posture:** `hl7-arrow` follows the workspace's "one job per crate, no opinions on others" principle. It does not pull in an async runtime, a specific I/O strategy, or a specific transport. It emits Arrow RecordBatches; consumers choose how to transport them (stdout, file, IPC socket, Flight, shared memory).

### HT-T01 — Audit current workspace for Arrow emission capability
- [x] **Status:** DONE (Apr 19, 2026)
- **Dependency:** —
- **Finding:** No Arrow emission present in any current crate. `hl7-mllp` frames bytes; `hl7-v2` produces a typed AST; `hl7-mindray` maps vendor codes; `fhir-r4` + `satusehat` produce JSON. None emit Apache Arrow RecordBatches. HT-T10 proceeds.
- **Notes:** Confirmed by inspection of `crates/*` tree and current README's Crates table. No code written.

### HT-T10 — Add `hl7-arrow` crate
- [ ] **Status:** BLOCKED (on HT-T10.1 schema design)
- **Dependency:** HT-T01 (DONE)
- **Notes:** New crate `crates/hl7-arrow/`. Depends on `hl7-v2` (for the parsed AST) and `arrow-rs` (for RecordBatch emission). No new dependency on `hl7-mllp`, `hl7-mindray`, `fhir-r4`, or `satusehat` — those are orthogonal. Keep the crate thin; `arrow-rs` does the heavy lifting for encoding, columnar layout, and IPC framing.

#### HT-T10.1 — Design Arrow schema for HL7 v2 messages
- [ ] **Status:** TODO
- **Dependency:** HT-T10
- **Notes:** One RecordBatch schema per message type (ADT, ORM, ORU, MDM, etc.). Schema design decisions:
  - **Type fidelity**: HL7 v2 is string-typed on the wire, but segments have implicit types (TS = timestamp, NM = numeric, etc.). Arrow schema surfaces these as proper `Timestamp`, `Float64`, `Utf8` etc.
  - **Nullability**: HL7 fields are often optional and/or repeat. Arrow `List<T>` for repeating fields; nullable columns for optional fields.
  - **Segment grouping**: Complex messages have grouped segments (e.g., ORU^R01 has OBR + 1..N OBX). RecordBatch layout uses struct/list columns to preserve the grouping.
  - **Schema versioning**: Arrow schema versions are **separate** from crate versions. Consumers pin against schema version; the crate can evolve internally without consumer break. Schema version is embedded as a metadata field on the RecordBatch schema.
  - **Neutral field naming**: use the canonical HL7 v2 segment/field identifiers (e.g., `msh.sending_application`, `obx.value_numeric`) rather than names specific to any downstream model. Consumers that want to map into a domain-specific schema (SNOMED, LOINC, FHIR observation codes, etc.) do that in their own code.
- **Documentation:** Write `crates/hl7-arrow/README.md` with: schema-version matrix, per-message-type field table, nullability/repetition rules, and schema metadata conventions. This README is the consumer contract.

#### HT-T10.2 — Emission examples
- [ ] **Status:** BLOCKED
- **Dependency:** HT-T10.1
- **Notes:** Two examples under `crates/hl7-arrow/examples/`:
  - **`mllp_listener.rs`** — TCP MLLP listener using `hl7-mllp` + `hl7-v2` + `hl7-arrow`; writes Arrow IPC stream to stdout. Intentionally minimal: single-connection, no reconnect logic, no graceful shutdown. Demonstration of the pattern, not production code.
  - **`pyarrow_consumer.py`** — Python companion reading the Arrow IPC stream from stdin. Uses `pyarrow` only. Demonstrates round-trip: Rust writes, Python reads, fields are typed correctly, nothing is lost in translation.
- **Kill condition:** Both examples must run end-to-end on a fixture HL7 v2 message before HT-T10 is considered complete.

#### HT-T10.3 — `hl7-arrow` crate metadata and publish readiness
- [ ] **Status:** BLOCKED
- **Dependency:** HT-T10.2
- **Notes:** Same metadata bar as other crates per T1.1: `description`, `repository`, `license` = `Apache-2.0`, `keywords` (hl7, healthcare, arrow, ipc), `categories` (encoding, parser-implementations). `#![forbid(unsafe_code)]`. docs.rs render check. Confirm `arrow-rs` and transitive deps clear `deny.toml`.

### HT-T20 — Document cross-language Arrow pattern in README
- [x] **Status:** DONE (Apr 19, 2026)
- **Dependency:** HT-T10 (design level — does not require code)
- **Notes:** New section "Cross-language integration via Apache Arrow" added to `README.md` after the Dependency Graph. Section covers: the pattern diagram (HL7 v2/MLLP source → Rust → Arrow IPC → any Arrow-capable consumer), rationale, transport-neutral options (stdout, file, Unix socket, Flight, shared memory), forward-referenced Rust + Python code example, and schema-versioning note (decoupled from crate version). Documentation satisfies HT-T20 kill-criterion even though the code it describes (HT-T10) is still BLOCKED.

### HT-T30 — First consumer feedback loop
- [ ] **Status:** BLOCKED
- **Dependency:** HT-T10.3
- **Notes:** After `hl7-arrow` v0.1.0 publishes to crates.io, solicit feedback from at least one real consumer wiring it into an actual HL7 ingestion pipeline. Feedback channels:
  - GitHub issues on this repo
  - Direct communication from maintainers of downstream projects that adopt the crate
- **Purpose of this task:** avoid the common trap of releasing a v0.1 that nobody has used end-to-end. "Works on my machine + passes CI" is not enough; at least one external integration must round-trip Arrow-encoded messages through a real HL7 source and a real downstream consumer before v0.1.0 is considered stable enough to tag `validated`.
- **Output:** a `crates/hl7-arrow/CHANGELOG.md` entry for v0.1.0 noting "validated end-to-end on <date>" once a consumer confirms the round-trip. No consumer names required — the validation note is a fact about the crate, not an endorsement or support claim for any specific downstream.

### v0.1 kill-criteria

All of the following must hold before `hl7-arrow` v0.1.0 is considered released:

1. **HT-T01 DONE** — audit recorded.
2. **HT-T10.1 DONE** — `hl7-arrow` crate exists; Arrow schema designed for at least ORU^R01 as the first reference message type; `crates/hl7-arrow/README.md` published with schema version matrix.
3. **HT-T10.2 DONE** — `mllp_listener.rs` + `pyarrow_consumer.py` examples run end-to-end on a fixture HL7 v2 message; round-trip verified (Rust writes → Python reads → same logical values).
4. **HT-T10.3 DONE** — crate metadata meets T1.1 bar; `cargo publish --dry-run -p hl7-arrow` passes; no warnings in `cargo doc -p hl7-arrow`; `deny.toml` clean.
5. **HT-T20 DONE** — README documents the cross-language Arrow pattern.
6. **Integration test green** — `cargo test --workspace` includes at least one round-trip test (Rust RecordBatch encode → deserialize → assert field equality). Python round-trip lives in `examples/`, not in CI, to avoid a Python toolchain requirement in this workspace's CI.
7. **cargo-deny clean** — `arrow-rs` and transitive deps clear the `deny.toml` allow-list.
8. **HT-T30 feedback** — at least one external integration has reported a successful end-to-end round-trip before the v0.1.0 release note claims `validated`.

---

## Phase 0 — Foundation `[workspace]` ✅

### T0.1 — Repository
- [x] Monorepo scaffolded under `crates/`
- [x] Workspace `Cargo.toml` with shared dependency table
- [x] `.gitignore` (target, Cargo.lock, .env)
- [x] Root `LICENSE` (Apache-2.0)
- [x] Root `README.md` with ecosystem overview and dependency graph
- [x] **Update copyright holder in all LICENSE files** — using "hl7-rs contributors" (standard for Apache-2.0)
- [x] Add `CONTRIBUTING.md` — contribution guide, DCO sign-off requirement
- [x] Add `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1)
- [x] Add `SECURITY.md` — vulnerability disclosure policy and contact

### T0.2 — CI baseline
- [x] `.github/workflows/ci.yml` — build, test, clippy, fmt on push/PR
  - [x] `actions/checkout@v4` → pinned to SHA `11bd71901bbe5b1630ceea73d27597364c9af683`
  - [x] `dtolnay/rust-toolchain@stable`
  - [x] `actions/cache@v4`
- [x] Add MSRV (Minimum Supported Rust Version) check job
  - [x] Install `rust-version` from workspace manifest
  - [x] Run `cargo +<msrv> check --workspace`

### T0.3 — Dependency hygiene
- [x] Add `cargo-deny` configuration (`deny.toml`)
  - [x] License allow-list: `Apache-2.0`, `MIT`, `MIT OR Apache-2.0`
  - [x] Reject `GPL`, `AGPL`, `LGPL`, `BUSL`
  - [x] Vulnerability advisories: deny all
  - [x] Duplicate dependency check: warn
- [x] Add `cargo-audit` to CI
  - [x] Run `cargo audit` on every push
  - [x] Add `audit.toml` for known false-positives

<!-- v0.1 note: When hl7-arrow is added (HT-T10), confirm arrow-rs and its transitive deps clear deny.toml. arrow-rs is Apache-2.0; expected clean. -->

---

## Phase 1 — Individual Crate v0.1.0

> Each crate ships `0.1.0` independently. See per-crate `TODO.md` for detailed tasks.
>
> *v0.1 addition:* `hl7-arrow` joins this phase as the sixth crate. Its own `crates/hl7-arrow/TODO.md` will be created under HT-T10.1.

### T1.1 — Publish readiness (all crates)
- [ ] Verify `cargo publish --dry-run -p <crate>` passes for each crate *(v0.1: now includes `hl7-arrow` per HT-T10.3)*
- [ ] Confirm all crates have `description`, `repository`, `license`, `keywords`, `categories`
- [ ] Confirm `README.md` renders correctly on docs.rs (no broken relative links)
- [ ] Remove intra-workspace `path =` deps from individual crates before publish (use `version =` only)

### T1.2 — Documentation
- [ ] `cargo doc --workspace --no-deps` — zero warnings *(v0.1: must include `hl7-arrow` once the crate exists)*
- [ ] Verify all public items have doc comments
- [ ] Add `#![doc = include_str!("../README.md")]` to each crate's `lib.rs`

### T1.3 — Test coverage
- [ ] Install `cargo-tarpaulin` (latest stable)
- [ ] Establish coverage baseline per crate (target: ≥ 70% for 0.1.0) *(v0.1: `hl7-arrow` held to the same ≥ 70% bar)*
- [ ] Add coverage report step to CI
  - [ ] `cargo tarpaulin --workspace --out Xml`
  - [ ] Upload to Codecov or similar

### T1.4 — MSRV
- [ ] Validate MSRV `1.75` is accurate for each crate
  - [ ] `cargo +1.75.0 check --workspace`
  - [ ] *(v0.1)* Verify `arrow-rs` MSRV compatibility with `1.75` when `hl7-arrow` lands. If `arrow-rs` requires a newer MSRV, either bump the workspace MSRV (documenting rationale) or gate `hl7-arrow` behind a higher MSRV annotation in its own `Cargo.toml`.
- [ ] Document MSRV policy in `CONTRIBUTING.md` (bump only on major/minor releases)

---

## Phase 2 — Integration & Hardening `[0.x.0 → pre-1.0]`

### T2.1 — Integration test suite
- [ ] Create `tests/` directory at workspace root
- [ ] Write end-to-end test: Mindray HL7 bytes → FHIR R4 JSON → SATUSEHAT-compliant output
  - [ ] Use real-world anonymized HL7 fixture files
  - [ ] Cover: ePM series message format
  - [ ] Cover: BeneVision N-series PDS format
  - [ ] Cover: iPM 9800 format
- [ ] *(v0.1)* Write end-to-end test: HL7 v2 bytes → `hl7-arrow` RecordBatch → deserialize → assert field equality (Rust-only round-trip; Python round-trip lives in `crates/hl7-arrow/examples/` per HT-T10.2, not in this CI)
- [ ] Write negative test: malformed MLLP frame → graceful error propagation
- [ ] Write negative test: unknown 99MNDRY codes → `VitalSign::Unknown` preserved
- [ ] *(v0.1)* Write negative test: HL7 v2 with unexpected segment repetitions → Arrow RecordBatch uses `List<T>` correctly; no silent truncation

### T2.2 — Fuzzing
- [ ] Add `cargo-fuzz` targets (requires nightly, run separately from CI)
  - [ ] `fuzz_mllp_decode` — fuzz `MllpFrame::decode`
  - [ ] `fuzz_hl7_parse` — fuzz `Hl7Message::parse`
  - [ ] *(v0.1)* `fuzz_hl7_arrow_encode` — fuzz `hl7_arrow::encode_*` functions (input: arbitrary parsed `Hl7Message`; output: valid RecordBatch or clean error)
- [ ] Document fuzz targets in `CONTRIBUTING.md`
- [ ] Run 10M iterations before each major release

### T2.3 — Benchmarks
- [ ] Add `benches/` using `criterion` (latest stable)
  - [ ] `bench_mllp_encode_decode` — throughput in MB/s
  - [ ] `bench_hl7_parse` — messages/second for typical ORU^R01
  - [ ] `bench_mindray_extract` — vitals extraction throughput
  - [ ] *(v0.1)* `bench_hl7_arrow_encode` — messages/second end-to-end (parse + encode to RecordBatch); target ≥ 10,000 msg/s on commodity hardware for ORU^R01 (revisit target after first measurement)
- [ ] Add benchmark CI job (compare against main branch)

### T2.4 — SemVer compatibility checks
- [ ] Add `cargo-semver-checks` to CI
  - [ ] Run on every PR that changes a crate
  - [ ] Block merge on breaking change without major version bump
  - [ ] *(v0.1)* Note: `hl7-arrow` Arrow *schema* version is tracked separately from its *crate* version. Crate-level SemVer checks won't catch schema-breaking changes. Add a secondary check: hash the emitted schema metadata on each PR and diff against main. Any diff requires an explicit schema-version bump documented in `crates/hl7-arrow/README.md`.

### T2.5 — Security
- [ ] Schedule weekly `cargo audit` via GitHub Actions `schedule:`
- [ ] Add `cargo-deny` CI check for license and advisory violations
- [ ] Review all `unwrap()` / `expect()` calls in non-test code — replace with proper errors
- [ ] Verify `#![forbid(unsafe_code)]` present in all crate roots *(v0.1: `hl7-arrow` must include this — arrow-rs internals use unsafe but the hl7-arrow public surface does not need to)*

---

## Phase 3 — Stable Release `[1.0.0]`

### T3.1 — Pre-1.0 audit
- [ ] External security review of public API surface
- [ ] API stability review — no planned breaking changes post-1.0
- [ ] Full documentation pass — every public item, every error variant
- [ ] CHANGELOGs complete from `0.0.1` through `1.0.0` *(v0.1: `hl7-arrow` CHANGELOG covers `0.1.0` through `1.0.0` — shorter history since the crate is newer)*

### T3.2 — Release tooling
- [ ] Add `cargo-release` configuration (`.cargo/release.toml`)
  - [ ] Release order: `hl7-mllp` → `hl7-v2` → `hl7-mindray` → `fhir-r4` → `satusehat`
  - [ ] *(v0.1 amendment)* Release order with `hl7-arrow`: `hl7-mllp` → `hl7-v2` → `hl7-mindray` → `hl7-arrow` → `fhir-r4` → `satusehat`. `hl7-arrow` releases after `hl7-v2` indexes (~1 min wait) and is independent of the `fhir-r4` / `satusehat` chain.
  - [ ] Auto-tag: `<crate>-v<version>`
  - [ ] Auto-update workspace dependency versions on release
- [ ] Add GitHub Release workflow
  - [ ] Triggered by tag push `<crate>-v*`
  - [ ] Auto-generate release notes from CHANGELOG
  - [ ] *(v0.1)* For `hl7-arrow` releases, the release note template includes the Arrow schema version(s) this crate version emits.
- [ ] Verify crates.io publish token scoped per-crate

### T3.3 — Post-release
- [ ] Submit crate announcements to `r/rust` and `users.rust-lang.org`
- [ ] Open issues for next cycle features
- [ ] Archive completed TODO items into `CHANGELOG.md`

---

## Toolchain Reference

| Tool | Purpose | Install |
|---|---|---|
| `cargo-audit` | CVE scanning | `cargo install cargo-audit` |
| `cargo-deny` | License + advisory policy | `cargo install cargo-deny` |
| `cargo-tarpaulin` | Coverage (Linux only) | `cargo install cargo-tarpaulin` |
| `cargo-semver-checks` | Breaking change detection | `cargo install cargo-semver-checks` |
| `cargo-release` | Automated release workflow | `cargo install cargo-release` |
| `cargo-fuzz` | Fuzz testing (requires nightly) | `cargo install cargo-fuzz` |
| `cargo-machete` | Unused dependency detection | `cargo install cargo-machete` |
| `cargo-skill` *(v0.1)* | Skill discovery / packaging for Cargo projects | `cargo install cargo-skill` |

---

## Version Log

| Version | Date | Description |
|---|---|---|
| v0.0 (implicit) | prior | Initial workspace TODO: Phases 0–3 defined. Five crates (`hl7-mllp`, `hl7-v2`, `hl7-mindray`, `fhir-r4`, `satusehat`) tracked. |
| v0.1 | Apr 19, 2026 | **v0.1 Scope — `hl7-arrow` crate added.** New section placed before Phase 0 (HT-T01 DONE, HT-T10 + 3 subtasks BLOCKED, HT-T20 DONE, HT-T30 BLOCKED, plus 8-point kill-criteria). `hl7-arrow` crate added as the sixth workspace crate with inline annotations across T0.3 (license), T1.1–T1.4 (publish/docs/coverage/MSRV), T2.1–T2.5 (integration tests, fuzzing, benchmarks, SemVer schema-decoupling note, unsafe posture), T3.1–T3.2 (CHANGELOG, release order amendment). `cargo-skill` added to Toolchain Reference. README.md updated in parallel: new crate row, Design Philosophy entry, Cross-language Apache Arrow section, Dependency Graph update, Quick Start deps note, Development publish order update, Status note. All prior content preserved verbatim. |

**Update discipline:** When a task changes status, append date. When a task is removed, tombstone it — no silent deletions.
