# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-09-03

### Added
- Integration test suite with 46 tests using `wiremock`
- Verification for all message types, error mapping, and webhook payloads

### Fixed
- Clippy warnings around error construction
- Corrected repository URL to point to `wiApi/rust`

## [0.1.0] - 2026-09-03

### Added
- Initial release of the official wi-api Rust SDK
- Asynchronous client powered by Tokio and Reqwest with rustls
- Strongly-typed WhatsApp session management and messaging models
- Webhook signature validation and parsing
