use bytes::BytesMut;
use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use hl7_mllp::MllpFrame;

/// Sample HL7 message payload for benchmarking.
const SAMPLE_PAYLOAD: &[u8] = b"MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240101120000||ORU^R01|12345|P|2.5\rPID|1||P001||Doe^John||19800101|M\rOBR|1|12345|LAB001|CBC^Complete Blood Count|||20240101120000\rOBX|1|NM|WBC^White Blood Cells||7.5|10*3/uL|4.0-10.0|N|||F\r";

fn encode_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");
    group.throughput(Throughput::Bytes(SAMPLE_PAYLOAD.len() as u64));

    group.bench_function("encode", |b| {
        b.iter(|| {
            let frame = MllpFrame::encode(black_box(SAMPLE_PAYLOAD));
            black_box(frame);
        });
    });

    // Reusable buffer for encode_into — demonstrates zero-allocation path
    let mut reuse_buf = BytesMut::with_capacity(SAMPLE_PAYLOAD.len() + 3);
    group.bench_function("encode_into", |b| {
        b.iter(|| {
            reuse_buf.clear();
            MllpFrame::encode_into(black_box(SAMPLE_PAYLOAD), &mut reuse_buf);
            black_box(&reuse_buf);
        });
    });

    group.finish();
}

fn decode_benchmark(c: &mut Criterion) {
    let framed = MllpFrame::encode(SAMPLE_PAYLOAD);
    let framed_bytes = framed.as_ref();

    let mut group = c.benchmark_group("decode");
    group.throughput(Throughput::Bytes(SAMPLE_PAYLOAD.len() as u64));

    group.bench_function("decode", |b| {
        b.iter(|| {
            let payload = MllpFrame::decode(black_box(framed_bytes)).unwrap();
            black_box(payload);
        });
    });

    group.finish();
}

fn roundtrip_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");
    group.throughput(Throughput::Bytes(SAMPLE_PAYLOAD.len() as u64));

    group.bench_function("encode_decode", |b| {
        b.iter(|| {
            let frame = MllpFrame::encode(black_box(SAMPLE_PAYLOAD));
            let payload = MllpFrame::decode(black_box(frame.as_ref())).unwrap();
            black_box(payload);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    encode_benchmark,
    decode_benchmark,
    roundtrip_benchmark
);
criterion_main!(benches);
