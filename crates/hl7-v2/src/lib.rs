//! # hl7-v2
//!
//! Zero-dependency HL7 v2 message parser.
//!
//! Parses raw HL7 v2 byte input into a typed AST without coupling to any
//! transport, runtime, or I/O library. Designed to be composed with
//! [`hl7-mllp`](https://crates.io/crates/hl7-mllp) for network use, or
//! used standalone for file/buffer parsing.
//!
//! ## Design
//!
//! - Zero dependencies — no alloc features required beyond `std`
//! - [`Hl7Message`]: top-level parsed message
//! - [`Segment`]: named segment (MSH, PID, OBR, OBX, ...)
//! - [`Field`]: field within a segment, with component/repetition access
//! - Strongly-typed accessors for common message types (ORU^R01, ADT^A01)
//!
//! ## Example
//!
//! ```rust
//! use hl7_v2::Hl7Message;
//!
//! let raw = b"MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240101120000||ORU^R01|12345|P|2.3.1\rOBX|1|NM|59408-5^SpO2^LN||98|%|95-100|N|||F";
//!
//! let msg = Hl7Message::parse(raw).unwrap();
//!
//! assert_eq!(msg.message_type(), Some("ORU^R01"));
//!
//! for obx in msg.segments("OBX") {
//!     // Access field values via raw_fields() to avoid lifetime constraints
//!     let value = obx.raw_fields().get(4).copied(); // OBX-5 (0-indexed from 0)
//!     println!("OBX value: {:?}", value);
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Errors produced during HL7 v2 parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The input was empty.
    Empty,
    /// The first segment was not MSH.
    MissingMsh,
    /// MSH segment was too short to extract encoding characters.
    MshTooShort,
    /// An unexpected encoding character was encountered.
    InvalidEncoding,
    /// UTF-8 decoding failed on a field value.
    Utf8Error,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "HL7 input is empty"),
            Self::MissingMsh => write!(f, "First segment is not MSH"),
            Self::MshTooShort => write!(f, "MSH segment too short to extract encoding characters"),
            Self::InvalidEncoding => write!(f, "Invalid encoding character in HL7 message"),
            Self::Utf8Error => write!(f, "UTF-8 decoding error in HL7 field"),
        }
    }
}

impl std::error::Error for ParseError {}

/// HL7 v2 encoding characters extracted from MSH-1 and MSH-2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodingChars {
    /// Field separator (default: `|`)
    pub field_sep: u8,
    /// Component separator (default: `^`)
    pub component_sep: u8,
    /// Repetition separator (default: `~`)
    pub repetition_sep: u8,
    /// Escape character (default: `\`)
    pub escape: u8,
    /// Sub-component separator (default: `&`)
    pub sub_component_sep: u8,
}

impl Default for EncodingChars {
    fn default() -> Self {
        Self {
            field_sep: b'|',
            component_sep: b'^',
            repetition_sep: b'~',
            escape: b'\\',
            sub_component_sep: b'&',
        }
    }
}

/// A single field within a segment, with component-level access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field<'a> {
    raw: &'a str,
    component_sep: u8,
}

impl<'a> Field<'a> {
    fn new(raw: &'a str, component_sep: u8) -> Self {
        Self { raw, component_sep }
    }

    /// The raw string value of the entire field.
    pub fn value(&self) -> &str {
        self.raw
    }

    /// Access a specific component within the field (1-indexed).
    pub fn component(&self, index: usize) -> Option<&str> {
        self.raw
            .split(self.component_sep as char)
            .nth(index.saturating_sub(1))
    }

    /// Returns true if the field value is empty or null (`""`).
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty() || self.raw == "\"\""
    }
}

/// A parsed HL7 v2 segment (e.g., MSH, PID, OBR, OBX).
#[derive(Debug, Clone)]
pub struct Segment<'a> {
    /// Segment name (e.g., "MSH", "OBX")
    pub name: &'a str,
    fields: Vec<&'a str>,
    encoding: EncodingChars,
}

impl<'a> Segment<'a> {
    /// Access a field by 1-based index.
    ///
    /// MSH is special: MSH-1 is the field separator itself.
    pub fn field(&self, index: usize) -> Option<Field<'_>> {
        self.fields
            .get(index.saturating_sub(1))
            .map(|raw| Field::new(raw, self.encoding.component_sep))
    }

    /// All fields in this segment as raw string slices.
    pub fn raw_fields(&self) -> &[&str] {
        &self.fields
    }
}

/// A fully parsed HL7 v2 message.
#[derive(Debug, Clone)]
pub struct Hl7Message<'a> {
    /// The encoding characters extracted from MSH.
    pub encoding: EncodingChars,
    segments: Vec<Segment<'a>>,
}

impl<'a> Hl7Message<'a> {
    /// Parse a raw HL7 v2 message from a byte slice.
    ///
    /// Segments are separated by carriage return (`\r`, 0x0D).
    /// Input may optionally be MLLP-stripped before passing here.
    pub fn parse(input: &'a [u8]) -> Result<Self, ParseError> {
        if input.is_empty() {
            return Err(ParseError::Empty);
        }

        let text = std::str::from_utf8(input).map_err(|_| ParseError::Utf8Error)?;

        // HL7 segments are delimited by \r (0x0D), sometimes \r\n
        let raw_segments: Vec<&str> = text
            .split('\r')
            .map(|s| s.trim_end_matches('\n'))
            .filter(|s| !s.is_empty())
            .collect();

        if raw_segments.is_empty() {
            return Err(ParseError::Empty);
        }

        let msh = raw_segments[0];
        if !msh.starts_with("MSH") {
            return Err(ParseError::MissingMsh);
        }
        if msh.len() < 8 {
            return Err(ParseError::MshTooShort);
        }

        // MSH-1 is the field separator (byte at index 3)
        let field_sep = msh.as_bytes()[3];

        // MSH-2 is the 4 encoding characters (bytes 4..8)
        let enc_bytes = &msh.as_bytes()[4..8];
        let encoding = EncodingChars {
            field_sep,
            component_sep: enc_bytes[0],
            repetition_sep: enc_bytes[1],
            escape: enc_bytes[2],
            sub_component_sep: enc_bytes[3],
        };

        let field_sep_char = field_sep as char;

        let segments = raw_segments
            .iter()
            .map(|raw_seg| {
                let mut fields: Vec<&str> = raw_seg.splitn(
                    usize::MAX,
                    field_sep_char,
                ).collect();

                let name = fields.remove(0);
                Segment {
                    name,
                    fields,
                    encoding: encoding.clone(),
                }
            })
            .collect();

        Ok(Self { encoding, segments })
    }

    /// Iterate over all segments with the given name (e.g., `"OBX"`).
    pub fn segments<'b>(&'b self, name: &'b str) -> impl Iterator<Item = &'b Segment<'b>> + 'b {
        self.segments.iter().filter(move |s| s.name == name)
    }

    /// Get the first segment with the given name.
    pub fn segment(&self, name: &str) -> Option<&Segment<'_>> {
        self.segments.iter().find(|s| s.name == name)
    }

    /// The MSH segment.
    pub fn msh(&self) -> Option<&Segment<'_>> {
        self.segment("MSH")
    }

    /// Message type string from MSH-9 (e.g., `"ORU^R01"`).
    pub fn message_type(&self) -> Option<&str> {
        let msh = self.msh()?;
        msh.fields.get(7).copied()
    }

    /// Message control ID from MSH-10.
    pub fn message_control_id(&self) -> Option<&str> {
        let msh = self.msh()?;
        msh.fields.get(8).copied()
    }

    /// HL7 version from MSH-12.
    pub fn version(&self) -> Option<&str> {
        let msh = self.msh()?;
        msh.fields.get(10).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_ORU: &[u8] = b"MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240101120000||ORU^R01|12345|P|2.3.1\rPID|1||P001^^^||Doe^John\rOBX|1|NM|59408-5^SpO2^LN||98|%|95-100|N|||F\rOBX|2|NM|8867-4^HR^LN||72|/min|60-100|N|||F";

    #[test]
    fn parse_message_type() {
        let msg = Hl7Message::parse(SAMPLE_ORU).unwrap();
        assert_eq!(msg.message_type(), Some("ORU^R01"));
    }

    #[test]
    fn parse_control_id() {
        let msg = Hl7Message::parse(SAMPLE_ORU).unwrap();
        assert_eq!(msg.message_control_id(), Some("12345"));
    }

    #[test]
    fn parse_obx_count() {
        let msg = Hl7Message::parse(SAMPLE_ORU).unwrap();
        assert_eq!(msg.segments("OBX").count(), 2);
    }

    #[test]
    fn parse_obx_value() {
        let msg = Hl7Message::parse(SAMPLE_ORU).unwrap();
        let first_obx = msg.segments("OBX").next().unwrap();
        // OBX-5 is the observation value
        let val = first_obx.raw_fields().get(4).copied();
        assert_eq!(val, Some("98"));
    }

    #[test]
    fn empty_input() {
        assert!(matches!(Hl7Message::parse(b""), Err(ParseError::Empty)));
    }

    #[test]
    fn missing_msh() {
        assert!(matches!(Hl7Message::parse(b"OBX|1|NM|..."), Err(ParseError::MissingMsh)));
    }
}
