use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use hl7_v2::Hl7Message;

/// Sample ORU^R01 message for benchmarking.
const SAMPLE_ORU: &[u8] = b"MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240101120000||ORU^R01|12345|P|2.3.1\rPID|1||P001^^^||Doe^John||19800101|M\rOBR|1|12345|LAB001|CBC^Complete Blood Count|||20240101120000\rOBX|1|NM|59408-5^SpO2^LN||98|%|95-100|N|||F\rOBX|2|NM|8867-4^HR^LN||72|/min|60-100|N|||F\rOBX|3|NM|9279-1^RR^LN||16|/min|12-20|N|||F";

fn parse_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");
    group.throughput(Throughput::Bytes(SAMPLE_ORU.len() as u64));

    group.bench_function("oru_r01", |b| {
        b.iter(|| {
            let msg = Hl7Message::parse(black_box(SAMPLE_ORU)).unwrap();
            black_box(msg);
        });
    });

    group.finish();
}

fn segment_lookup_benchmark(c: &mut Criterion) {
    let msg = Hl7Message::parse(SAMPLE_ORU).unwrap();

    let mut group = c.benchmark_group("segment_lookup");
    group.throughput(Throughput::Elements(1));

    group.bench_function("segments_obx", |b| {
        b.iter(|| {
            let count = black_box(&msg).segments("OBX").count();
            black_box(count);
        });
    });

    group.finish();
}

criterion_group!(benches, parse_benchmark, segment_lookup_benchmark);
criterion_main!(benches);
