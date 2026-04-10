# Security Policy

## Supported Versions

The following versions of hl7-rs crates are currently supported with security updates:

| Crate | Version | Supported |
|-------|---------|-----------|
| hl7-mllp | 0.0.1 | ✅ |
| hl7-v2 | 0.0.1 | ✅ |
| hl7-mindray | 0.0.1 | ✅ |
| fhir-r4 | 0.0.1 | ✅ |
| satusehat | 0.0.1 | ✅ |

As the project matures, we will maintain security updates for the latest minor version of each major release.

## Reporting a Vulnerability

If you discover a security vulnerability in hl7-rs, please report it responsibly via GitHub Security Advisory or by opening a private issue.

**Urgent contact**: [X @ks_sha888](https://x.com/ks_sha888)

Please include:
- A description of the vulnerability
- Steps to reproduce (if applicable)
- Potential impact assessment
- Suggested fix (if any)

## Response Timeline

We aim to respond to security reports within:

- **48 hours**: Initial acknowledgment
- **7 days**: Assessment and response plan
- **30 days**: Resolution or workaround provided

For critical vulnerabilities (RCE, data exfiltration), we will prioritize faster response times.

## Disclosure Policy

We follow responsible disclosure:

1. Reporter submits vulnerability privately
2. We investigate and confirm the issue
3. We develop and test a fix
4. We coordinate release with reporter
5. We publish security advisory and CVE (if applicable)
6. We publicly acknowledge the reporter's contribution (with permission)

## Security Best Practices

When using hl7-rs in production:

- Validate all input data before processing
- Use TLS for all network communications
- Keep dependencies updated (`cargo audit`)
- Run with minimal required privileges
- Enable logging for security-relevant events
- Regularly review access controls

## Security Features

This workspace provides:

- `#![forbid(unsafe_code)]` in all crate roots
- Input validation for HL7 message parsing
- Bounds checking on all buffer operations
- Fuzz testing targets for critical parsers

## Acknowledgments

We thank security researchers who responsibly disclose vulnerabilities. Past acknowledgments will be listed here.
