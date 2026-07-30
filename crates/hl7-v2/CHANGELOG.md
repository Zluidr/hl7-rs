# Changelog

All notable changes this project will be documented in this file.

format is based on [Keep Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-30

### Added

- **`Hl7Message::all_segments()`**: iterate every segment in original document order, regardless of name — needed to reconstruct hierarchical groupings (e.g. `OBR`/`OBX` ORDER_OBSERVATION structure) that `segments(name)` alone loses by filtering to a single segment type

### Fixed

- **`Hl7Message::segments()`**: corrected an inconsistent lifetime signature (`impl Iterator<Item = &'b Segment<'_>> + 'b` → `impl Iterator<Item = &'b Segment<'b>> + 'b`)

## [0.0.1] - 2025-01-01

### Added

- Initial placeholder release
