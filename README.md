# wi-api Rust SDK

[![Crates.io](https://img.shields.io/crates/v/wi-api?style=flat-square)](https://crates.io/crates/wi-api)
[![docs.rs](https://img.shields.io/docsrs/wi-api?style=flat-square)](https://docs.rs/wi-api)
[![license](https://img.shields.io/crates/l/wi-api?style=flat-square)](LICENSE)

Rust SDK for [wi-api](https://wi.api.br). Async, fully typed, no OpenSSL dependency.

## Install

```toml
[dependencies]
wi-api = "0.1"
tokio = { version = "1", features = ["full"] }
```

Or with cargo-add:

```bash
cargo add wi-api
```

## Quick start

```rust
use wi_api::{Wi, types::SendTextParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wi = Wi::new(std::env::var("WI_API_KEY")?);

    let msg = wi
        .session("my-instance")
        .send_text(SendTextParams {
            to: "5511999999999".into(),
            text: "Hello from wi-api".into(),
            ..Default::default()
        })
        .await?;

    println!("sent: {}", msg.message_id);
    Ok(())
}
```

## Sessions

```rust
let session = wi.session("my-instance");

// Start connection flow
session.connect().await?;

// Get QR code (base64 PNG)
let qr = session.qr().await?;
println!("{}", qr.qr.unwrap_or_default());

// Pair by phone number
let result = session.pair_phone("5511999999999").await?;
println!("{}", result.pair_code.unwrap_or_default());

// Check status
let status = session.status().await?;
println!("connected={} phone={:?}", status.connected, status.phone);

session.disconnect().await?;
session.logout().await?;
```

## Sending messages

```rust
use wi_api::types::*;

session.send_text(SendTextParams {
    to: "5511999999999".into(),
    text: "Hello!".into(),
    ..Default::default()
}).await?;

session.send_image(SendImageParams {
    to: "5511999999999".into(),
    url: "https://example.com/photo.jpg".into(),
    caption: Some("Look at this".into()),
    ..Default::default()
}).await?;

session.send_audio(SendAudioParams {
    to: "5511999999999".into(),
    url: "https://example.com/audio.ogg".into(),
    ptt: Some(true),
    ..Default::default()
}).await?;

session.send_document(SendDocumentParams {
    to: "5511999999999".into(),
    url: "https://example.com/report.pdf".into(),
    filename: Some("report.pdf".into()),
    ..Default::default()
}).await?;

session.send_location(SendLocationParams {
    to: "5511999999999".into(),
    latitude: -23.5505,
    longitude: -46.6333,
    title: Some("São Paulo".into()),
    ..Default::default()
}).await?;

session.react(SendReactionParams {
    to: "5511999999999".into(),
    message_id: message_id.into(),
    emoji: "".into(),
    ..Default::default()
}).await?;
```

## Webhooks

```rust
use wi_api::webhook::{verify_signature, parse_event};

// Axum handler
async fn webhook(
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> axum::response::Response {
    use axum::{http::StatusCode, response::IntoResponse};

    let sig = headers
        .get("x-wi-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let secret = std::env::var("WI_WEBHOOK_SECRET").unwrap();

    if !verify_signature(&body, sig, secret.as_bytes()) {
        return (StatusCode::UNAUTHORIZED, "invalid signature").into_response();
    }

    let event = parse_event(&body).unwrap();
    println!("event: {} session: {}", event.event, event.session_id);

    StatusCode::NO_CONTENT.into_response()
}
```

Using `WebhookPayload`:

```rust
use wi_api::webhook::WebhookPayload;

let secret = std::env::var("WI_WEBHOOK_SECRET").unwrap();
match WebhookPayload::from_parts(body, sig, secret.as_bytes()) {
    None => StatusCode::UNAUTHORIZED.into_response(),
    Some(Err(e)) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    Some(Ok(p)) => {
        println!("event: {}", p.event.event);
        StatusCode::NO_CONTENT.into_response()
    }
}
```

## Error handling

```rust
use wi_api::error::WiError;

match session.send_text(params).await {
    Ok(msg) => println!("sent: {}", msg.message_id),
    Err(WiError::Api { status, message, .. }) => eprintln!("API {status}: {message}"),
    Err(WiError::Request(e)) => eprintln!("network: {e}"),
    Err(e) => eprintln!("{e}"),
}
```

## Configuration

```rust
use std::time::Duration;

let wi = Wi::builder(api_key)
    .base_url("https://endpoint.wi.api.br")
    .timeout(Duration::from_secs(10))
    .build();
```

## Resources

- [crates.io](https://crates.io/crates/wi-api)
- [docs.rs](https://docs.rs/wi-api)
- [Dashboard](https://wi.api.br)
- [Docs](https://docs.wi.api.br)
- [Changelog](https://github.com/wiApi/rust/releases)
