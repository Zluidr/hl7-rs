# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`hl7-rs` is a Cargo workspace of small, composable Rust crates for healthcare data integration: HL7 v2 transport/parsing, vendor field mapping, FHIR R4 resources, and an Indonesian SATUSEHAT profile on top. Each crate does one job and has no opinion on the others' runtime, transport, or framework choices — don't add cross-crate coupling (e.g. don't make `hl7-v2` depend on an async runtime, don't make `hl7-mllp` know about FHIR).

```
hl7-mllp        — frame bytes in/out (MLLP transport envelope), nothing else
hl7-v2          — parse bytes → typed AST, zero dependencies
hl7-mindray     — map Mindray 99MNDRY vendor codes → VitalSign enum (depends on hl7-v2)
fhir-r4         — FHIR R4 resource structs + builders (serde/serde_json only)
satusehat       — Indonesian SATUSEHAT FHIR profile + API client (depends on fhir-r4)
hl7-arrow       — planned (v0.1, not yet implemented) — Arrow RecordBatch emission from hl7-v2 AST
```

Dependency direction: `satusehat → fhir-r4`, `hl7-mindray → hl7-v2`. `hl7-mllp`, `hl7-v2`, and `fhir-r4` have no intra-workspace dependencies.

## Commands

```bash
# Build / test everything (CI runs with --all-features)
cargo build --workspace --all-features
cargo test --workspace --all-features

# Single crate
cargo test -p hl7-mllp
cargo test -p hl7-v2

# Single test
cargo test -p hl7-mllp some_test_name -- --exact

# Lint + format (CI fails on any clippy warning)
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

# MSRV check (workspace MSRV is read from Cargo.toml's rust-version)
cargo +1.94 check --workspace

# Supply-chain checks (also run as pre-commit hooks when Cargo.toml/Cargo.lock/deny.toml or audit.toml change)
cargo deny check
cargo audit

# hl7-mllp benchmarks (criterion, the only crate with a bench target)
cargo bench -p hl7-mllp
```

Pre-commit hooks (`.pre-commit-config.yaml`) run fmt, clippy, `cargo test --workspace --all-features`, `cargo deny check`, and `cargo audit` — expect CI to fail on the same things pre-commit would catch locally.

Publish order matters because of intra-workspace path deps (also documented in `CONTRIBUTING.md`): `hl7-mllp` → `hl7-v2` → `hl7-mindray` (wait ~1 min for hl7-v2 to index) → `fhir-r4` → `satusehat` (wait ~1 min for fhir-r4 to index).

## Architecture notes

**`hl7-v2`** (`crates/hl7-v2/src/lib.rs`) is the foundation: zero-dependency, borrows from the input buffer rather than copying. `Hl7Message<'a>::parse(bytes)` → `Segment<'a>` → `Field<'a>`, all lifetime-tied to the original byte slice. `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`. Field access has an ergonomics/lifetime tradeoff — prefer `raw_fields()` over deep-indexing chains when lifetime constraints get in the way (see the doc example in the crate).

**`hl7-mllp`** (`crates/hl7-mllp/src/lib.rs`, ~1300 lines, the most feature-gated crate) provides three abstractions: `MllpFrame` (stateless one-shot encode/decode, zero-allocation `decode()` returning a slice into the input), `MllpFramer` (stateful streaming accumulator for chunked network I/O), and `MllpTransport`/`AsyncMllpTransport` traits for callers to implement their own transport (TCP, serial, in-memory, etc.). Feature flags:
- `std` (default) — enables std-dependent pieces
- `no_std` mode (`--no-default-features`) still requires `alloc` (uses `BytesMut` internally) — not for allocator-less targets
- `async` — adds `AsyncMllpTransport` (tokio, `io-util`+`net` only, no full runtime dep)
- `timestamps` — adds `chrono`-based helpers
- `noncompliance` — opt-in handling for real-world MLLP deviations from spec; keep this off the default path since it encodes deliberate spec violations

When touching this crate, run the full feature matrix (`--all-features` and default-only) since behavior is feature-gated throughout — a change that compiles under `--all-features` can still break `no_std`/no-`async` builds.

**`hl7-mindray`** (`crates/hl7-mindray/src/lib.rs`) maps vendor-specific OBX segments (Mindray's `99MNDRY` private code space) from a parsed `Hl7Message` into a typed `VitalSign` enum via `MindrayOru::from_message`. Vendor code constants live in the nested `codes` module.

**`fhir-r4`** (`crates/fhir-r4/src/lib.rs`) is split by resource: `observation.rs`, `patient.rs`, `types.rs`, plus an optional `satusehat.rs` module gated behind the `satusehat` feature (profile-specific extensions living in the resource crate itself, separate from the `satusehat` crate's own API-client concerns). Builder pattern for resource construction (e.g. `ObservationBuilder`).

**`satusehat`** (`crates/satusehat/src/lib.rs`) wraps `fhir-r4` resources with Indonesia's SATUSEHAT national health platform profile: `SatuSehatConfig`/`SatuSehatEnv` (sandbox vs production endpoints), OAuth `AccessToken`, and `observation.rs` for profile-specific wrapping (e.g. `SatuSehatObservation::from_observation`). The `client` feature adds an optional `reqwest` dependency for the actual HTTP calls — without it, the crate only does data modeling/serialization.

## Conventions to follow

- `#![forbid(unsafe_code)]` and `#![warn(missing_docs)]` at the top of every crate's `lib.rs` — match this in new crates/modules.
- Public APIs need rustdoc; crate-level docs use the `//! # crate-name` + Design + Example pattern already present in each `lib.rs`.
- Shared dependency versions are pinned once in the workspace root `Cargo.toml` (`[workspace.dependencies]`) — add new shared deps there and reference with `{ workspace = true }` rather than pinning per-crate.
- `deny.toml` only allows Apache-2.0/MIT/BSD-2/BSD-3/ISC/Unicode-DFS-2016/Unicode-3.0/OpenSSL licenses and crates.io as the only source — check before adding a new dependency.
- Commits require DCO sign-off (`git commit -s`) per `CONTRIBUTING.md`.
- Per-crate `TODO.md` files (`crates/*/TODO.md`) track crate-specific work; the root `TODO.md` tracks cross-crate/workspace-level work (currently the `hl7-arrow` v0.1 effort).

## Behavioral guidelines

Source: [andrej-karpathy-skills/CLAUDE.md](https://github.com/multica-ai/andrej-karpathy-skills/blob/main/CLAUDE.md). These bias toward caution over speed — for trivial tasks, use judgment.

### 1. Think Before Coding

Don't assume. Don't hide confusion. Surface tradeoffs.

- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

### 2. Simplicity First

Minimum code that solves the problem. Nothing speculative.

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

### 3. Surgical Changes

Touch only what you must. Clean up only your own mess.

- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it — don't delete it.
- Remove imports/variables/functions that YOUR changes made unused; don't remove pre-existing dead code unless asked.

The test: every changed line should trace directly to the user's request.

### 4. Goal-Driven Execution

Define success criteria. Loop until verified.

- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

## Commit & documentation standards

**Do not include `Co-Authored-By:` trailers in commit messages.** This applies to all assistant-generated commits, including those produced by Claude Code or any other AI tool. Commit attribution stays with the human author. Boilerplate trailers add noise to the history without conveying meaningful authorship and have been retroactively stripped from past commits.

**English-only requirement:**

- All `Plans.md` content must be in English (headers, table columns, task descriptions, status markers).
- No Japanese characters in `Plans.md` status markers (use `cc:done` instead of `cc:完了`, `cc:wip` instead of `cc:WIP`, etc).
- All harness output and documentation must be in English.
- This applies strictly to tracked files; commit to this constraint when editing `Plans.md`.
