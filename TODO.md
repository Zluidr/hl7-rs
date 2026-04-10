# TODO — hl7-rs workspace

Tracking all cross-crate work. Crate-specific tasks live in each crate's own `TODO.md`.

Version targets follow [SemVer](https://semver.org). Phase gates map to crate versions.

---

## Phase 0 — Foundation `[workspace]` ✅

### T0.1 — Repository
- [x] Monorepo scaffolded under `crates/`
- [x] Workspace `Cargo.toml` with shared dependency table
- [x] `.gitignore` (target, Cargo.lock, .env)
- [x] Root `LICENSE` (Apache-2.0)
- [x] Root `README.md` with ecosystem overview and dependency graph
- [ ] **Update copyright holder in all LICENSE files** — currently placeholder; replace before first public release
- [ ] Add `CONTRIBUTING.md` — contribution guide, DCO sign-off requirement
- [ ] Add `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1)
- [ ] Add `SECURITY.md` — vulnerability disclosure policy and contact

### T0.2 — CI baseline
- [x] `.github/workflows/ci.yml` — build, test, clippy, fmt on push/PR
- [ ] Pin action versions to exact SHA (security hardening)
  - [ ] `actions/checkout@v4` → pin to SHA
  - [ ] `dtolnay/rust-toolchain@stable` → pin to SHA
  - [ ] `actions/cache@v4` → pin to SHA
- [ ] Add MSRV (Minimum Supported Rust Version) check job
  - [ ] Install `rust-version` from workspace manifest
  - [ ] Run `cargo +<msrv> check --workspace`

### T0.3 — Dependency hygiene
- [ ] Add `cargo-deny` configuration (`deny.toml`)
  - [ ] License allow-list: `Apache-2.0`, `MIT`, `MIT OR Apache-2.0`
  - [ ] Reject `GPL`, `AGPL`, `LGPL`, `BUSL`
  - [ ] Vulnerability advisories: deny all
  - [ ] Duplicate dependency check: warn
- [ ] Add `cargo-audit` to CI
  - [ ] Run `cargo audit` on every push
  - [ ] Add `audit.toml` for known false-positives

---

## Phase 1 — Individual Crate v0.1.0

> Each crate ships `0.1.0` independently. See per-crate `TODO.md` for detailed tasks.

### T1.1 — Publish readiness (all crates)
- [ ] Verify `cargo publish --dry-run -p <crate>` passes for each crate
- [ ] Confirm all crates have `description`, `repository`, `license`, `keywords`, `categories`
- [ ] Confirm `README.md` renders correctly on docs.rs (no broken relative links)
- [ ] Remove intra-workspace `path =` deps from individual crates before publish (use `version =` only)

### T1.2 — Documentation
- [ ] `cargo doc --workspace --no-deps` — zero warnings
- [ ] Verify all public items have doc comments
- [ ] Add `#![doc = include_str!("../README.md")]` to each crate's `lib.rs`

### T1.3 — Test coverage
- [ ] Install `cargo-tarpaulin` (latest stable)
- [ ] Establish coverage baseline per crate (target: ≥ 70% for 0.1.0)
- [ ] Add coverage report step to CI
  - [ ] `cargo tarpaulin --workspace --out Xml`
  - [ ] Upload to Codecov or similar

### T1.4 — MSRV
- [ ] Validate MSRV `1.75` is accurate for each crate
  - [ ] `cargo +1.75.0 check --workspace`
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
- [ ] Write negative test: malformed MLLP frame → graceful error propagation
- [ ] Write negative test: unknown 99MNDRY codes → `VitalSign::Unknown` preserved

### T2.2 — Fuzzing
- [ ] Add `cargo-fuzz` targets (requires nightly, run separately from CI)
  - [ ] `fuzz_mllp_decode` — fuzz `MllpFrame::decode`
  - [ ] `fuzz_hl7_parse` — fuzz `Hl7Message::parse`
- [ ] Document fuzz targets in `CONTRIBUTING.md`
- [ ] Run 10M iterations before each major release

### T2.3 — Benchmarks
- [ ] Add `benches/` using `criterion` (latest stable)
  - [ ] `bench_mllp_encode_decode` — throughput in MB/s
  - [ ] `bench_hl7_parse` — messages/second for typical ORU^R01
  - [ ] `bench_mindray_extract` — vitals extraction throughput
- [ ] Add benchmark CI job (compare against main branch)

### T2.4 — SemVer compatibility checks
- [ ] Add `cargo-semver-checks` to CI
  - [ ] Run on every PR that changes a crate
  - [ ] Block merge on breaking change without major version bump

### T2.5 — Security
- [ ] Schedule weekly `cargo audit` via GitHub Actions `schedule:`
- [ ] Add `cargo-deny` CI check for license and advisory violations
- [ ] Review all `unwrap()` / `expect()` calls in non-test code — replace with proper errors
- [ ] Verify `#![forbid(unsafe_code)]` present in all crate roots

---

## Phase 3 — Stable Release `[1.0.0]`

### T3.1 — Pre-1.0 audit
- [ ] External security review of public API surface
- [ ] API stability review — no planned breaking changes post-1.0
- [ ] Full documentation pass — every public item, every error variant
- [ ] CHANGELOGs complete from `0.0.1` through `1.0.0`

### T3.2 — Release tooling
- [ ] Add `cargo-release` configuration (`.cargo/release.toml`)
  - [ ] Release order: `hl7-mllp` → `hl7-v2` → `hl7-mindray` → `fhir-r4` → `satusehat`
  - [ ] Auto-tag: `<crate>-v<version>`
  - [ ] Auto-update workspace dependency versions on release
- [ ] Add GitHub Release workflow
  - [ ] Triggered by tag push `<crate>-v*`
  - [ ] Auto-generate release notes from CHANGELOG
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
