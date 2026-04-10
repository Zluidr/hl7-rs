# hl7-mindray

[![Crates.io](https://img.shields.io/crates/v/hl7-mindray.svg)](https://crates.io/crates/hl7-mindray)
[![Docs.rs](https://docs.rs/hl7-mindray/badge.svg)](https://docs.rs/hl7-mindray)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**Mindray patient monitor HL7 field mappings.**

Extracts typed vital signs from Mindray HL7 v2 `ORU^R01` messages, handling both standard LOINC/MDC observation codes and Mindray's private `99MNDRY` code space.

---

## Supported Devices

| Device | Protocol | HL7 Version |
|---|---|---|
| BeneVision N-series (N17, N15, N12) | PDS over TCP/MLLP | 2.3.1 |
| ePM series (ePM 10/12/15) | Direct LAN HL7 output | 2.x |
| iPM 9800 | HL7 LAN (wired/wireless) | 2.x |

---

## Usage

```rust
use hl7_v2::Hl7Message;
use hl7_mindray::{MindrayOru, VitalSign};

// Raw HL7 from Mindray device (MLLP-stripped)
let raw = b"MSH|^~\\&|BeneVision|ICU1|EMR||20240101120000||ORU^R01|001|P|2.3.1\r\
            OBX|1|NM|59408-5^SpO2^LN||98|%|95-100|N|||F\r\
            OBX|2|NM|8867-4^HR^LN||72|/min|60-100|N|||F\r\
            OBX|3|NM|9279-1^RR^LN||16|/min|12-20|N|||F";

let msg = Hl7Message::parse(raw)?;
let oru = MindrayOru::from_message(&msg)?;

println!("SpO2: {:?}%", oru.spo2());           // Some(98.0)
println!("HR:   {:?} bpm", oru.heart_rate());  // Some(72.0)
println!("RR:   {:?} /min", oru.respiratory_rate()); // Some(16.0)

// Iterate all extracted vitals
for vital in oru.vitals() {
    match vital {
        VitalSign::SpO2(v)            => println!("SpO2: {v}%"),
        VitalSign::HeartRate(v)       => println!("HR: {v} bpm"),
        VitalSign::RespiratoryRate(v) => println!("RR: {v}/min"),
        VitalSign::Temperature(v)     => println!("Temp: {v}°C"),
        VitalSign::Nibp { systolic, diastolic, .. } => {
            println!("NIBP: {systolic}/{diastolic} mmHg")
        }
        VitalSign::Unknown { code, value, .. } => {
            println!("Unknown code {code}: {value}")
        }
        _ => {}
    }
}
```

---

## Code Space Coverage

| Parameter | Code | Source |
|---|---|---|
| SpO2 | `59408-5` | LOINC |
| Heart rate | `8867-4` | LOINC |
| Respiratory rate | `9279-1` | LOINC |
| Temperature | `8310-5` | LOINC |
| NIBP sys/dia/mean | `99MNDRY-NIBP-*` | Mindray private |
| EtCO2 | `99MNDRY-ETCO2` | Mindray private |
| IBP channels 1–4 | `99MNDRY-IBP*` | Mindray private |

Unknown codes are preserved in `VitalSign::Unknown` rather than silently dropped.

---

## Ecosystem

| Crate | Purpose |
|---|---|
| [`hl7-mllp`](https://crates.io/crates/hl7-mllp) | MLLP transport framing |
| [`hl7-v2`](https://crates.io/crates/hl7-v2) | HL7 v2 parser |
| [`hl7-mindray`](https://crates.io/crates/hl7-mindray) | Mindray field mappings (this crate) |
| [`fhir-r4`](https://crates.io/crates/fhir-r4) | FHIR R4 resource types |
| [`satusehat`](https://crates.io/crates/satusehat) | Indonesian SATUSEHAT FHIR profile |

---

## Status

`0.0.1` — initial placeholder. Active development in progress.

## License

Apache-2.0
