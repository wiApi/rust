use bytes::Bytes;
use wi_api::webhook::{parse_event, verify_signature, WebhookPayload};

const SECRET: &[u8] = b"test-webhook-secret-32-chars!!!";
const PAYLOAD: &str = r#"{"event":"message","session_id":"sess-1","data":{"id":"msg_1"},"timestamp":"2026-09-03T00:00:00Z"}"#;

fn make_sig(body: &[u8], secret: &[u8]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret).unwrap();
    mac.update(body);
    let result = mac.finalize().into_bytes();
    format!("sha256={}", hex::encode(result))
}

#[test]
fn verify_valid_signature() {
    let sig = make_sig(PAYLOAD.as_bytes(), SECRET);
    assert!(verify_signature(PAYLOAD.as_bytes(), &sig, SECRET));
}

#[test]
fn verify_invalid_signature() {
    let bad = "sha256=0000000000000000000000000000000000000000000000000000000000000000";
    assert!(!verify_signature(PAYLOAD.as_bytes(), bad, SECRET));
}

#[test]
fn verify_tampered_body() {
    let sig = make_sig(PAYLOAD.as_bytes(), SECRET);
    let tampered = b"tampered body";
    assert!(!verify_signature(tampered, &sig, SECRET));
}

#[test]
fn verify_wrong_scheme() {
    let sig = make_sig(PAYLOAD.as_bytes(), SECRET);
    // strip_prefix("sha256=") won't match "sha512=..." so returns false
    let wrong_scheme = sig.replace("sha256=", "sha512=");
    assert!(!verify_signature(PAYLOAD.as_bytes(), &wrong_scheme, SECRET));
}

#[test]
fn verify_empty_signature() {
    assert!(!verify_signature(PAYLOAD.as_bytes(), "", SECRET));
}

#[test]
fn verify_wrong_secret() {
    let sig = make_sig(PAYLOAD.as_bytes(), SECRET);
    assert!(!verify_signature(PAYLOAD.as_bytes(), &sig, b"wrong-secret"));
}

#[test]
fn verify_malformed_hex() {
    // Valid prefix but non-hex content — hex::decode will fail, returns false
    assert!(!verify_signature(PAYLOAD.as_bytes(), "sha256=not-valid-hex!!!", SECRET));
}

#[test]
fn parse_event_valid() {
    let event = parse_event(PAYLOAD.as_bytes()).unwrap();
    assert_eq!(event.event, "message");
    assert_eq!(event.session_id, "sess-1");
    assert_eq!(event.timestamp, "2026-09-03T00:00:00Z");
}

#[test]
fn parse_event_invalid_json() {
    let result = parse_event(b"not-json");
    assert!(result.is_err());
}

#[test]
fn parse_event_missing_fields() {
    // Missing required fields causes a deserialization error
    let result = parse_event(b"{}");
    assert!(result.is_err());
}

#[test]
fn webhook_payload_from_parts_valid() {
    let body = Bytes::from_static(PAYLOAD.as_bytes());
    let sig = make_sig(&body, SECRET);
    let result = WebhookPayload::from_parts(body, &sig, SECRET);
    assert!(result.is_some());
    let inner = result.unwrap();
    assert!(inner.is_ok());
    let payload = inner.unwrap();
    assert_eq!(payload.event.event, "message");
    assert_eq!(payload.event.session_id, "sess-1");
}

#[test]
fn webhook_payload_from_parts_raw_preserved() {
    let body = Bytes::from_static(PAYLOAD.as_bytes());
    let sig = make_sig(&body, SECRET);
    let payload = WebhookPayload::from_parts(body.clone(), &sig, SECRET)
        .unwrap()
        .unwrap();
    assert_eq!(payload.raw, body);
}

#[test]
fn webhook_payload_from_parts_invalid_sig() {
    let body = Bytes::from_static(PAYLOAD.as_bytes());
    // None means invalid signature — caller should respond 401
    let result = WebhookPayload::from_parts(body, "sha256=bad", SECRET);
    assert!(result.is_none());
}

#[test]
fn webhook_payload_from_parts_invalid_json() {
    let body = Bytes::from_static(b"sha256=valid-prefix-but-bad-json");
    let sig = make_sig(&body, SECRET);
    let result = WebhookPayload::from_parts(body, &sig, SECRET);
    // Signature matches, but JSON parse fails → Some(Err(...))
    assert!(result.is_some());
    assert!(result.unwrap().is_err());
}

#[test]
fn header_constant() {
    assert_eq!(WebhookPayload::HEADER, "x-wi-signature");
}
