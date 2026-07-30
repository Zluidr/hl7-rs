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
//!
//! ## Example
//!
//! ```
//! use hl7_arrow::encode_oru_r01;
//! use hl7_v2::Hl7Message;
//!
//! let raw = b"MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240115103000||ORU^R01|MSG00001|P|2.3.1\r\
//! PID|1||P12345^^^MRN||Doe^Jane||19800615|F\r\
//! OBR|1||ORD001|99MNDRY^SpotCheckVitals||20240115102000|20240115102000\r\
//! OBX|1|NM|59408-5^SpO2^LN||97|%|95-100|N|||F\r";
//! let message = Hl7Message::parse(raw).unwrap();
//! let batch = encode_oru_r01(&message).unwrap();
//! assert_eq!(batch.num_rows(), 1);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod datetime;
mod encode;
mod schema;

pub use encode::{EncodeError, encode_oru_r01};
pub use schema::oru_r01_schema;
