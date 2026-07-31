//! Negative tests (T2.1): malformed MLLP frames must be rejected with the
//! specific `MllpError` variant, not panic or silently return garbage.

use hl7_mllp::{MllpError, MllpFrame};

#[test]
fn rejects_frame_missing_start_byte() {
    // No leading VT (0x0B) — starts directly with payload.
    let mut frame = b"MSH|^~\\&|test".to_vec();
    frame.push(0x1c); // FS
    frame.push(0x0d); // CR

    assert_eq!(MllpFrame::decode(&frame), Err(MllpError::MissingStartByte));
}

#[test]
fn rejects_frame_missing_end_sequence() {
    // VT prefix present, but no trailing FS+CR.
    let mut frame = vec![0x0b];
    frame.extend_from_slice(b"MSH|^~\\&|test");

    assert_eq!(
        MllpFrame::decode(&frame),
        Err(MllpError::MissingEndSequence)
    );
}

#[test]
fn rejects_truncated_frame_as_incomplete() {
    // Shorter than the minimum possible frame (VT + 1 byte + FS + CR).
    // This is also the shortest buffer that could otherwise represent an
    // empty payload (VT immediately followed by FS+CR); the `buf.len() < 4`
    // guard fires first, so `MllpError::EmptyPayload` is unreachable via
    // `MllpFrame::decode` as currently implemented. Discovered while writing
    // this test (T2.1); out of scope to fix here, see Plans.md version log.
    let frame = vec![0x0b, 0x1c, 0x0d];

    assert_eq!(MllpFrame::decode(&frame), Err(MllpError::Incomplete));
}
