# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `QEntitiesParseOptions::max_key_length()` for limiting the maximum length of keys while parsing.
- `QEntitiesParseOptions::max_value_length()` for limiting the maximum length of values while parsing.
- `QEntitiesParseOptions::max_entities()` for limiting the maximum number of entities while parsing.
- `QEntitiesParseOptions::max_entity_key_values()` for limiting the maximum number of key-value pairs an entity can have while parsing.
- `PartialEq` and `Eq` trait implementations too `QEntitiesParserLocation`.

## [0.2.2] - 2023-08-08

### Fixed
- Incorrect location being reported for unterminated C style comments.

## [0.2.1] - 2023-08-07

### Fixed
- Incorrect location being reported for unterminated quoted strings.

## [0.2.0] - 2023-08-04

### Added
- `CHANGELOG.md`.
- `QEntities::get_unchecked()`.
- `QEntityRef::get_unchecked()`.

### Removed
- Unnecessary `raw` feature dependency from `hashbrown`.

## [0.1.0] - 2023-08-02

[unreleased]: https://github.com/IanE9/qentities/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/IanE9/qentities/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/IanE9/qentities/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/IanE9/qentities/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/IanE9/qentities/releases/tag/v0.1.0
