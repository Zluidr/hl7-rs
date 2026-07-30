#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod observation;
pub mod patient;
pub mod types;

#[cfg(feature = "satusehat")]
pub mod satusehat;
