#!/usr/bin/env python3
"""Read an Arrow IPC stream produced by the `mllp_listener` Rust example
(HT-T10.2) from stdin, and print its schema plus each RecordBatch's row
count and field values.

Demonstrates the round-trip half of the pattern: Rust encodes an HL7 v2
ORU^R01 message to Arrow and writes it as an IPC stream, `pyarrow` reads it
back with no data loss and no custom deserialization code, only the
`pyarrow` package.

Usage:
    cargo run -p hl7-arrow --example mllp_listener > /tmp/out.arrow &
    # send an MLLP-framed HL7 v2 message to 127.0.0.1:2575
    python3 crates/hl7-arrow/examples/pyarrow_consumer.py < /tmp/out.arrow
"""

import sys

import pyarrow as pa
import pyarrow.ipc as ipc


def main() -> int:
    reader = ipc.open_stream(sys.stdin.buffer)

    metadata = reader.schema.metadata or {}
    schema_version = metadata.get(b"hl7_arrow.schema_version", b"?").decode()
    message_type = metadata.get(b"hl7_arrow.message_type", b"?").decode()
    print(f"hl7_arrow.schema_version: {schema_version}")
    print(f"hl7_arrow.message_type: {message_type}")
    print(reader.schema)

    total_rows = 0
    for batch in reader:
        total_rows += batch.num_rows
        print(f"--- batch: {batch.num_rows} row(s) ---")
        for row in pa.Table.from_batches([batch]).to_pylist():
            print(row)

    print(f"total rows: {total_rows}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
