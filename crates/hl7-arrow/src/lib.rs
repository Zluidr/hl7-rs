//! # hl7-arrow
//!
//! Apache Arrow `RecordBatch` emission for parsed HL7 v2 messages.
//!
//! Turns a typed `hl7_v2::Hl7Message` AST into columnar Arrow data, with no
//! opinion on transport (stdout, file, socket, Flight, shared memory). See
//! [`crates/hl7-arrow/README.md`](https://github.com/Zluidr/hl7-rs/blob/main/crates/hl7-arrow/README.md)
//! for the ORU^R01 schema design (HT-T10.1) that this crate implements.
//!
//! ## Design
//!
//! This crate depends only on `hl7-v2` and `arrow` — no other intra-workspace
//! crate — so it stays usable independently of transport (`hl7-mllp`), vendor
//! mapping (`hl7-mindray`), or FHIR (`fhir-r4`/`satusehat`) concerns.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
