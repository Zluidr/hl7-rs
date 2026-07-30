use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use hl7_mindray::MindrayOru;
use hl7_v2::Hl7Message;

/// Sample Mindray ORU^R01 message (BeneVision-style) for benchmarking.
const SAMPLE: &[u8] = b"MSH|^~\\&|BeneVision|ICU1|EMR||20240101120000||ORU^R01|001|P|2.3.1\rOBX|1|NM|59408-5^SpO2^LN||98|%|95-100|N|||F\rOBX|2|NM|8867-4^HR^LN||72|/min|60-100|N|||F\rOBX|3|NM|9279-1^RR^LN||16|/min|12-20|N|||F\rOBX|4|NM|8310-5^Temp^LN||36.8|Cel|36.0-37.5|N|||F\rOBX|5|ST|99MNDRY-NIBP-SYS^NIBP Sys^99MNDRY||120|mmHg|90-140|N|||F";

fn extract_benchmark(c: &mut Criterion) {
    let msg = Hl7Message::parse(SAMPLE).unwrap();

    let mut group = c.benchmark_group("extract");
    group.throughput(Throughput::Bytes(SAMPLE.len() as u64));

    group.bench_function("from_message", |b| {
        b.iter(|| {
            let oru = MindrayOru::from_message(black_box(&msg)).unwrap();
            black_box(oru);
        });
    });

    group.finish();
}

fn end_to_end_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");
    group.throughput(Throughput::Bytes(SAMPLE.len() as u64));

    group.bench_function("parse_and_extract", |b| {
        b.iter(|| {
            let msg = Hl7Message::parse(black_box(SAMPLE)).unwrap();
            let oru = MindrayOru::from_message(&msg).unwrap();
            black_box(oru);
        });
    });

    group.finish();
}

criterion_group!(benches, extract_benchmark, end_to_end_benchmark);
criterion_main!(benches);
