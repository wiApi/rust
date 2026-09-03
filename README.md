# wi-api — Rust SDK

[![Crates.io](https://img.shields.io/crates/v/wi-api?style=flat-square&color=0d9373)](https://crates.io/crates/wi-api)
[![docs.rs](https://img.shields.io/docsrs/wi-api?style=flat-square)](https://docs.rs/wi-api)
[![license](https://img.shields.io/crates/l/wi-api?style=flat-square&color=0d9373)](LICENSE)

Official Rust SDK for the [wi-api](https://wi.api.br) WhatsApp platform.

- Async/await with `tokio`
- Fully typed with `serde`
- `rustls` by default — no OpenSSL dependency
- Builder pattern for configuration
- Feature-gated `axum` and `actix-web` webhook extractors

---

## Install

```toml
[dependencies]
wi-api = "0.1"
tokio = { version = "1", features = ["full"] }
```

---

## Quick start

```rust
use wi_api::{Wi, types::SendTextParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wi = Wi::new(std::env::var("WI_API_KEY")?);
    let session = wi.session("my-instance");

    let msg = session.send_text(SendTextParams {
        to: "5511999999999".into(),
        text: "Hello from wi-api".into(),
        ..Default::default()
    }).await?;

    println!("sent: {}", msg.message_id);
    Ok(())
}
```

---

## Sessions

```rust
// Connect — starts QR or pairphone flow
session.connect().await?;

// Get QR code (base64 PNG)
let qr = session.qr().await?;
println!("{}", qr.qr.unwrap_or_default());

// Pair by phone number
let result = session.pair_phone("5511999999999").await?;
println!("{}", result.pair_code.unwrap_or_default()); // 8-char code

// Status
let status = session.status().await?;
println!("connected={} phone={:?}", status.connected, status.phone);

// Disconnect / logout
session.disconnect().await?;
session.logout().await?;
```

---

## Sending messages

```rust
use wi_api::types::*;

// Text
session.send_text(SendTextParams {
    to: "5511999999999".into(),
    text: "Hello!".into(),
    ..Default::default()
}).await?;

// Image
session.send_image(SendImageParams {
    to: "5511999999999".into(),
    url: "https://example.com/photo.jpg".into(),
    caption: Some("Check this out".into()),
    ..Default::default()
}).await?;

// Voice note
session.send_audio(SendAudioParams {
    to: "5511999999999".into(),
    url: "https://example.com/audio.ogg".into(),
    ptt: Some(true),
    ..Default::default()
}).await?;

// Document
session.send_document(SendDocumentParams {
    to: "5511999999999".into(),
    url: "https://example.com/report.pdf".into(),
    filename: Some("Q3-report.pdf".into()),
    ..Default::default()
}).await?;

// Location
session.send_location(SendLocationParams {
    to: "5511999999999".into(),
    latitude: -23.5505,
    longitude: -46.6333,
    title: Some("São Paulo".into()),
    ..Default::default()
}).await?;

// Reaction
session.react(SendReactionParams {
    to: "5511999999999".into(),
    message_id: "MESSAGE_ID".into(),
    emoji: "👍".into(),
    ..Default::default()
}).await?;
```

---

## Webhooks

```rust
use wi_api::webhook::{verify_signature, parse_event};

// Axum handler example
async fn webhook(
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> axum::response::Response {
    let sig = headers
        .get("x-wi-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let secret = std::env::var("WI_WEBHOOK_SECRET").unwrap();

    if !verify_signature(&body, sig, secret.as_bytes()) {
        return (axum::http::StatusCode::UNAUTHORIZED, "invalid signature").into_response();
    }

    let event = match parse_event(&body) {
        Ok(e) => e,
        Err(_) => return (axum::http::StatusCode::BAD_REQUEST, "bad json").into_response(),
    };

    match event.event.as_str() {
        "message" => {
            let msg: wi_api::types::IncomingMessage =
                serde_json::from_str(event.data.get()).unwrap();
            println!("[{}] {}: {}", msg.chat, msg.from, msg.text.unwrap_or_default());
        }
        "connected" => println!("session {} connected", event.session_id),
        _ => {}
    }

    (axum::http::StatusCode::NO_CONTENT, "").into_response()
}
```

### Using `WebhookPayload`

```rust
use wi_api::webhook::WebhookPayload;

// Returns None if signature invalid — respond 401
let secret = std::env::var("WI_WEBHOOK_SECRET").unwrap();
let payload = WebhookPayload::from_parts(body, sig, secret.as_bytes());

match payload {
    None => (StatusCode::UNAUTHORIZED).into_response(),
    Some(Err(e)) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    Some(Ok(p)) => {
        println!("event: {}", p.event.event);
        StatusCode::NO_CONTENT.into_response()
    }
}
```

---

## Error handling

```rust
use wi_api::error::WiError;

match session.send_text(params).await {
    Ok(msg) => println!("sent: {}", msg.message_id),
    Err(WiError::Api { status, message, .. }) => {
        eprintln!("API error {status}: {message}");
    }
    Err(WiError::Request(e)) => {
        eprintln!("network error: {e}");
    }
    Err(e) => eprintln!("error: {e}"),
}
```

---

## Configuration

```rust
use std::time::Duration;

let wi = Wi::builder(api_key)
    .base_url("https://endpoint.wi.api.br")
    .timeout(Duration::from_secs(10))
    .build();
```

---

## License

MIT — [wi.api.br](https://wi.api.br)
