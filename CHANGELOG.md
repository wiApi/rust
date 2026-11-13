# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-11-13

### Added
- `types::PairingFailed`, `PairingSucceeded`, `PairingMethod` and `PairingFailureReason` for the `pairing.failed` / `pairing.succeeded` events: `reason` is the closed set (`PairingFailureReason::KNOWN`) and a value a newer server adds decodes as `PairingFailureReason::Unknown` instead of failing; treat it like `Error`
- `types::Disconnected` and `DisconnectReason` for the `disconnected` event: `will_reconnect` is always present, `reason` is the closed set (`DisconnectReason::KNOWN`) and a value a newer server adds decodes as `DisconnectReason::Unknown` instead of failing
- `webhook::verify_webhook` / `VerifyOptions` / `WebhookPayload::from_headers`: verify `x-wi-signature-v2` (HMAC of `<t>.<body>`) with replay protection (default tolerance 300 s); fall back to the legacy `x-wi-signature` only when no v2 header is sent
- `types::Receipt` and `ReceiptStatus` for the `receipt` event

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
