# TODO — hl7-mllp

Transport-agnostic MLLP framing for HL7 v2 messages.

---

## Phase 0 — Foundation `[0.0.1]` ✅

- [x] Crate scaffolded with `Cargo.toml`, `README.md`, `LICENSE`
- [x] `MllpFrame` — encode, decode, find_frame_end, build_ack
- [x] `MllpTransport` trait — transport-agnostic boundary
- [x] `MllpError` enum with `Display` + `Error` impl
- [x] 5 unit tests passing
- [x] 1 doctest passing
- [x] `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`
- [x] Workspace dependency integration (`bytes = { workspace = true }`)

---

## Phase 1 — Spec-Complete Implementation `[0.1.0]`

### T1.1 — MLLP specification compliance
- [x] Verify byte sequence against HL7 v2.5.1 Appendix C (MLLP spec)
  - [x] Confirm VT=0x0B, FS=0x1C, CR=0x0D are correct per spec
  - [x] Confirm single-byte start-block only (no multi-byte variants)
- [x] Handle consecutive frames in a single buffer
  - [x] `find_all_frames(buf: &[u8]) -> Vec<(usize, usize)>` — returns (start, end) pairs
  - [x] Test: two back-to-back frames in one buffer
  - [x] Test: partial third frame at end of buffer
- [x] Handle non-compliant senders (optional feature `noncompliance`)
  - [x] Tolerate missing final CR after FS
  - [x] Tolerate extra bytes before VT start byte
  - [x] Feature-gate: `#[cfg(feature = "noncompliance")]`

### T1.2 — ACK generation
- [x] Replace `chrono_now_str()` stub with real timestamp
  - [x] Add `chrono` as optional dependency behind `timestamps` feature
  - [x] Default: caller provides timestamp string
  - [x] With `timestamps` feature: auto-generate HL7 DTM format (`YYYYMMDDHHmmss`)
- [x] `build_ack` — validate `message_control_id` is non-empty
- [x] Add `build_nack(message_control_id: &str, error_code: &str, error_text: &str) -> Option<String>`
- [x] Test: round-trip ACK parse (ACK is valid HL7 that `hl7-v2` can parse)

### T1.3 — Streaming support
- [x] `MllpFramer` struct — stateful streaming frame accumulator
  - [x] `fn push(&mut self, bytes: &[u8])` — append bytes to internal buffer
  - [x] `fn next_frame(&mut self) -> Option<Vec<u8>>` — pop next complete frame
  - [x] `fn is_empty(&self) -> bool`
  - [x] Internal buffer: `BytesMut` (already a dep)
- [x] Test: push bytes in 1-byte increments, recover complete frame
- [x] Test: push two frames in one call, recover both

### T1.4 — Error improvements
- [x] Add `MllpError::InvalidFrame { reason: String }` for future use
- [x] Implement `From<MllpError> for std::io::Error` (for transport impls)
- [x] All error variants — verify messages are actionable (no "unexpected error")

### T1.5 — Documentation
- [x] Module-level doc: explain MLLP framing in plain language with ASCII diagram
- [x] `MllpFrame::encode` — document exact output byte layout
- [x] `MllpFrame::decode` — document zero-copy guarantee (lifetime note)
- [x] `MllpFramer` — document streaming usage with example
- [x] `MllpTransport` — document expected contract (thread safety, error handling)
- [x] Add `examples/tcp_listener.rs` — minimal blocking TCP MLLP listener

### T1.6 — Tests
- [ ] Test: `encode` then `decode` round-trip with Unicode payload
- [ ] Test: `decode` on minimum-length valid frame (1-byte payload)
- [ ] Test: `find_frame_end` with buffer containing exactly one frame
- [ ] Test: `find_frame_end` with empty buffer
- [ ] Test: `MllpFramer` push/pop streaming

---

## Phase 2 — Async & Ecosystem `[0.2.0]`

### T2.1 — Async transport trait (optional)
- [ ] Add `AsyncMllpTransport` trait behind `async` feature flag
  - [ ] `async fn read_frame(&mut self) -> Result<Vec<u8>, Self::Error>`
  - [ ] `async fn write_frame(&mut self, frame: &[u8]) -> Result<(), Self::Error>`
  - [ ] Requires `async-trait` or AFIT (Rust 1.75+ native async fn in trait)
- [ ] Add `examples/tokio_tcp.rs` — tokio TCP implementation of `AsyncMllpTransport`

### T2.2 — `no_std` compatibility
- [ ] Audit: identify `std`-only dependencies
  - [ ] `bytes` crate — check `no_std` support (it has `alloc` feature)
  - [ ] `std::error::Error` — gated behind `std` feature
- [ ] Add `default-features = false` path for embedded targets
- [ ] Test compile with `--no-default-features` + `alloc`

### T2.3 — Performance
- [ ] Add `criterion` benchmark: `encode` + `decode` throughput (MB/s)
- [ ] Profile allocation pattern — `encode` currently allocates `BytesMut`
  - [ ] Consider `encode_into(payload: &[u8], buf: &mut BytesMut)` zero-alloc variant

---

## Phase 3 — Stable `[1.0.0]`

### T3.1 — API freeze
- [ ] Final review of public API — no planned changes after 1.0
- [ ] `cargo semver-checks` — confirm no accidental breaking changes from 0.1.0
- [ ] Deprecation notices for any pre-1.0 items being removed

### T3.2 — Quality gates
- [ ] `cargo doc --no-deps` — zero warnings
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo fmt --check` — passes
- [ ] `cargo audit` — zero vulnerabilities
- [ ] Test coverage ≥ 80%
- [ ] CHANGELOG complete

### T3.3 — Release
- [ ] Tag `hl7-mllp-v1.0.0`
- [ ] Publish to crates.io
- [ ] GitHub Release with notes
