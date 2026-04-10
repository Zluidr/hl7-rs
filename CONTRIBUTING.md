# Contributing to hl7-rs

Thank you for your interest in contributing to hl7-rs! This document provides guidelines for contributing to this workspace.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Create a new branch for your contribution
4. Make your changes
5. Submit a pull request

## Development Setup

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --workspace --all-targets --all-features

# Check formatting
cargo fmt --check
```

## Minimum Supported Rust Version (MSRV)

This workspace requires Rust 1.75 or later. MSRV bumps are only done on major or minor releases, not patch releases.

## Contribution Guidelines

### Code Style

- Follow Rust idioms and standard formatting (`cargo fmt`)
- Address all clippy warnings before submitting
- Write clear, concise commit messages
- Add documentation for all public APIs

### Testing

- Add tests for new functionality
- Ensure all tests pass before submitting PR
- Include integration tests where appropriate

### Documentation

- Use rustdoc conventions for API documentation
- Update README.md if adding new features
- Add examples for complex functionality

## Developer Certificate of Origin (DCO)

By contributing to this project, you certify that:

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.


Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

### Sign-off

All commits must be signed off. Use `git commit -s` to automatically add the sign-off:

```bash
git commit -s -m "feat: add new HL7 segment parser"
```

This adds a `Signed-off-by` line to your commit message:
```
feat: add new HL7 segment parser

Signed-off-by: Your Name <your.email@example.com>
```

## Pull Request Process

1. Ensure your PR description clearly describes the problem and solution
2. Reference any related issues using `Fixes #123` or `Closes #456`
3. Ensure CI passes (build, test, clippy, fmt)
4. Request review from maintainers
5. Address review feedback promptly

## Release Process

Releases follow Semantic Versioning (SemVer). The release order is:

1. `hl7-mllp`
2. `hl7-v2`
3. `hl7-mindray`
4. `fhir-r4`
5. `satusehat`

## Questions?

Feel free to open an issue for discussion before starting work on significant changes.
