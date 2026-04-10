//! # hl7-mindray
//!
//! Mindray patient monitor HL7 field mappings.
//!
//! Mindray devices (BeneVision N-series, ePM series, iPM 9800) output HL7 v2
//! `ORU^R01` messages using a mix of standard LOINC/MDC codes and Mindray's
//! private `99MNDRY` code space. This crate maps both code spaces to a
//! unified [`VitalSign`] enum for downstream use.
//!
//! ## Supported Devices
//!
//! - BeneVision N-series (N17, N15, N12) — PDS protocol, HL7 v2.3.1
//! - ePM series (ePM 10/12/15) — direct HL7 LAN output
//! - iPM 9800 — HL7 v2 over wired/wireless LAN
//!
//! ## Example
//!
//! ```rust
//! use hl7_v2::Hl7Message;
//! use hl7_mindray::MindrayOru;
//!
//! let raw = b"MSH|^~\\&|BeneVision|ICU1|EMR||20240101120000||ORU^R01|001|P|2.3.1\r\
//!             OBX|1|NM|59408-5^SpO2^LN||98|%|95-100|N|||F\r\
//!             OBX|2|NM|8867-4^HR^LN||72|/min|60-100|N|||F";
//!
//! let msg = Hl7Message::parse(raw).unwrap();
//! let oru = MindrayOru::from_message(&msg).unwrap();
//!
//! for vital in oru.vitals() {
//!     println!("{:?}", vital);
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use hl7_v2::Hl7Message;

/// A vital sign value extracted from a Mindray HL7 ORU^R01 message.
#[derive(Debug, Clone, PartialEq)]
pub enum VitalSign {
    /// Heart rate in beats per minute.
    HeartRate(f64),
    /// Peripheral oxygen saturation (SpO2) as a percentage.
    SpO2(f64),
    /// Respiratory rate in breaths per minute.
    RespiratoryRate(f64),
    /// Non-invasive blood pressure: systolic / diastolic (mmHg).
    Nibp {
        /// Systolic pressure in mmHg.
        systolic: f64,
        /// Diastolic pressure in mmHg.
        diastolic: f64,
        /// Mean arterial pressure in mmHg.
        mean: Option<f64>,
    },
    /// Body temperature in degrees Celsius.
    Temperature(f64),
    /// End-tidal CO2 in mmHg.
    EtCO2(f64),
    /// Invasive blood pressure on the specified channel.
    Ibp {
        /// IBP channel number (1–4).
        channel: u8,
        /// Systolic pressure in mmHg.
        systolic: f64,
        /// Diastolic pressure in mmHg.
        diastolic: f64,
        /// Mean arterial pressure in mmHg.
        mean: Option<f64>,
    },
    /// A parameter identified by code but not yet mapped to a typed variant.
    Unknown {
        /// The raw OBX-3 code string.
        code: String,
        /// The raw OBX-5 value string.
        value: String,
        /// The raw OBX-6 unit string.
        unit: Option<String>,
    },
}

/// LOINC and 99MNDRY codes used by Mindray devices.
///
/// Mindray uses standard LOINC codes where available and falls back to
/// their private `99MNDRY` code space for device-specific parameters.
/// Reference: Mindray Patient Data Share Protocol Programmer's Guide.
pub mod codes {
    // Standard LOINC codes used by Mindray
    /// SpO2 — LOINC 59408-5
    pub const SPO2_LOINC: &str = "59408-5";
    /// Heart rate — LOINC 8867-4
    pub const HEART_RATE_LOINC: &str = "8867-4";
    /// Respiratory rate — LOINC 9279-1
    pub const RESP_RATE_LOINC: &str = "9279-1";
    /// Body temperature — LOINC 8310-5
    pub const TEMPERATURE_LOINC: &str = "8310-5";

    // 99MNDRY private codes (Mindray-specific)
    /// NIBP systolic — 99MNDRY private
    pub const NIBP_SYS_MNDRY: &str = "99MNDRY-NIBP-SYS";
    /// NIBP diastolic — 99MNDRY private
    pub const NIBP_DIA_MNDRY: &str = "99MNDRY-NIBP-DIA";
    /// NIBP mean — 99MNDRY private
    pub const NIBP_MEAN_MNDRY: &str = "99MNDRY-NIBP-MEAN";
    /// EtCO2 — 99MNDRY private
    pub const ETCO2_MNDRY: &str = "99MNDRY-ETCO2";
    /// IBP channel 1 systolic
    pub const IBP1_SYS_MNDRY: &str = "99MNDRY-IBP1-SYS";
    /// IBP channel 1 diastolic
    pub const IBP1_DIA_MNDRY: &str = "99MNDRY-IBP1-DIA";
    /// IBP channel 1 mean
    pub const IBP1_MEAN_MNDRY: &str = "99MNDRY-IBP1-MEAN";
}

/// Errors from Mindray ORU parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MindrayError {
    /// The message type was not ORU^R01.
    NotOru,
    /// A required OBX field was missing.
    MissingField,
    /// A numeric value could not be parsed.
    InvalidNumeric(String),
}

impl std::fmt::Display for MindrayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotOru => write!(f, "Message is not ORU^R01"),
            Self::MissingField => write!(f, "Required OBX field is missing"),
            Self::InvalidNumeric(s) => write!(f, "Cannot parse numeric value: {s}"),
        }
    }
}

impl std::error::Error for MindrayError {}

/// A parsed Mindray ORU^R01 observation message.
pub struct MindrayOru {
    vitals: Vec<VitalSign>,
}

impl MindrayOru {
    /// Parse vital signs from a Mindray HL7 ORU^R01 message.
    pub fn from_message(msg: &Hl7Message<'_>) -> Result<Self, MindrayError> {
        if msg.message_type() != Some("ORU^R01") {
            return Err(MindrayError::NotOru);
        }

        let mut vitals = Vec::new();

        for obx in msg.segments("OBX") {
            // OBX-3: observation identifier (code^text^system)
            // Extract first component (the code) from the raw field string
            let code = obx
                .field(3)
                .map(|f| f.value().split('^').next().unwrap_or("").to_string())
                .unwrap_or_default();

            // OBX-5: observation value
            let value_str = obx
                .field(5)
                .map(|f| f.value().to_string())
                .unwrap_or_default();

            // OBX-6: units
            let unit = obx.field(6).map(|f| f.value().to_string());

            if value_str.is_empty() {
                continue;
            }

            let vital = parse_obx_to_vital(&code, &value_str, unit);
            vitals.push(vital);
        }

        Ok(Self { vitals })
    }

    /// All vital signs extracted from this message.
    pub fn vitals(&self) -> &[VitalSign] {
        &self.vitals
    }

    /// Find the first occurrence of a specific vital sign type.
    pub fn heart_rate(&self) -> Option<f64> {
        self.vitals.iter().find_map(|v| match v {
            VitalSign::HeartRate(hr) => Some(*hr),
            _ => None,
        })
    }

    /// SpO2 value if present.
    pub fn spo2(&self) -> Option<f64> {
        self.vitals.iter().find_map(|v| match v {
            VitalSign::SpO2(s) => Some(*s),
            _ => None,
        })
    }

    /// Respiratory rate if present.
    pub fn respiratory_rate(&self) -> Option<f64> {
        self.vitals.iter().find_map(|v| match v {
            VitalSign::RespiratoryRate(r) => Some(*r),
            _ => None,
        })
    }
}

fn parse_obx_to_vital(code: &str, value: &str, unit: Option<String>) -> VitalSign {
    use codes::*;

    let numeric = value.parse::<f64>();

    match code {
        SPO2_LOINC => numeric
            .map(VitalSign::SpO2)
            .unwrap_or_else(|_| unknown(code, value, unit)),

        HEART_RATE_LOINC => numeric
            .map(VitalSign::HeartRate)
            .unwrap_or_else(|_| unknown(code, value, unit)),

        RESP_RATE_LOINC => numeric
            .map(VitalSign::RespiratoryRate)
            .unwrap_or_else(|_| unknown(code, value, unit)),

        TEMPERATURE_LOINC => numeric
            .map(VitalSign::Temperature)
            .unwrap_or_else(|_| unknown(code, value, unit)),

        ETCO2_MNDRY => numeric
            .map(VitalSign::EtCO2)
            .unwrap_or_else(|_| unknown(code, value, unit)),

        _ => unknown(code, value, unit),
    }
}

fn unknown(code: &str, value: &str, unit: Option<String>) -> VitalSign {
    VitalSign::Unknown {
        code: code.to_string(),
        value: value.to_string(),
        unit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &[u8] = b"MSH|^~\\&|BeneVision|ICU1|EMR||20240101120000||ORU^R01|001|P|2.3.1\rOBX|1|NM|59408-5^SpO2^LN||98|%|95-100|N|||F\rOBX|2|NM|8867-4^HR^LN||72|/min|60-100|N|||F\rOBX|3|NM|9279-1^RR^LN||16|/min|12-20|N|||F";

    #[test]
    fn parses_spo2() {
        let msg = Hl7Message::parse(SAMPLE).unwrap();
        let oru = MindrayOru::from_message(&msg).unwrap();
        assert_eq!(oru.spo2(), Some(98.0));
    }

    #[test]
    fn parses_heart_rate() {
        let msg = Hl7Message::parse(SAMPLE).unwrap();
        let oru = MindrayOru::from_message(&msg).unwrap();
        assert_eq!(oru.heart_rate(), Some(72.0));
    }

    #[test]
    fn parses_respiratory_rate() {
        let msg = Hl7Message::parse(SAMPLE).unwrap();
        let oru = MindrayOru::from_message(&msg).unwrap();
        assert_eq!(oru.respiratory_rate(), Some(16.0));
    }
}
