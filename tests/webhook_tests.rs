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
    assert!(!verify_signature(
        PAYLOAD.as_bytes(),
        "sha256=not-valid-hex!!!",
        SECRET
    ));
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

// ─── verify_webhook (v2: signed timestamp) ────────────────────────────────────

use wi_api::types::{Receipt, ReceiptStatus};
use wi_api::webhook::{verify_webhook, VerifyOptions};

const NOW: i64 = 1_800_000_000;

fn make_sig_v2(body: &[u8], secret: &[u8], ts: i64) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
    mac.update(format!("{ts}.").as_bytes());
    mac.update(body);
    format!("t={ts},v1={}", hex::encode(mac.finalize().into_bytes()))
}

fn at(now: i64) -> VerifyOptions {
    VerifyOptions {
        now: Some(now),
        ..VerifyOptions::default()
    }
}

#[test]
fn verify_webhook_accepts_fresh_v2_case_insensitive() {
    let sig = make_sig_v2(PAYLOAD.as_bytes(), SECRET, NOW);
    let headers = [("X-WI-Signature-V2", sig.as_str())];
    assert!(verify_webhook(
        PAYLOAD.as_bytes(),
        &headers,
        SECRET,
        at(NOW + 299)
    ));
}

#[test]
fn verify_webhook_rejects_replay_outside_tolerance() {
    let sig = make_sig_v2(PAYLOAD.as_bytes(), SECRET, NOW);
    let headers = [("x-wi-signature-v2", sig.as_str())];
    assert!(!verify_webhook(
        PAYLOAD.as_bytes(),
        &headers,
        SECRET,
        at(NOW + 301)
    ));
    assert!(!verify_webhook(
        PAYLOAD.as_bytes(),
        &headers,
        SECRET,
        at(NOW - 301)
    ));
    let strict = VerifyOptions {
        tolerance_seconds: 30,
        now: Some(NOW + 60),
    };
    assert!(!verify_webhook(
        PAYLOAD.as_bytes(),
        &headers,
        SECRET,
        strict
    ));
    let lenient = VerifyOptions {
        tolerance_seconds: 120,
        now: Some(NOW + 60),
    };
    assert!(verify_webhook(
        PAYLOAD.as_bytes(),
        &headers,
        SECRET,
        lenient
    ));
}

#[test]
fn verify_webhook_rejects_tampered_timestamp_body_and_secret() {
    let sig = make_sig_v2(PAYLOAD.as_bytes(), SECRET, NOW);
    let v1 = sig.split_once(',').unwrap().1;
    let moved = format!("t={},{v1}", NOW + 1);
    assert!(!verify_webhook(
        PAYLOAD.as_bytes(),
        &[("x-wi-signature-v2", moved.as_str())],
        SECRET,
        at(NOW)
    ));
    assert!(!verify_webhook(
        b"tampered",
        &[("x-wi-signature-v2", sig.as_str())],
        SECRET,
        at(NOW)
    ));
    assert!(!verify_webhook(
        PAYLOAD.as_bytes(),
        &[("x-wi-signature-v2", sig.as_str())],
        b"wrong",
        at(NOW)
    ));
}

#[test]
fn verify_webhook_malformed_v2_never_falls_back_to_legacy() {
    let legacy = make_sig(PAYLOAD.as_bytes(), SECRET);
    for v2 in [
        "",
        "garbage",
        "t=abc,v1=00",
        "t=1800000000",
        "v1=00",
        "t=1800000000,v1=zz",
    ] {
        let headers = [
            ("x-wi-signature-v2", v2),
            ("x-wi-signature", legacy.as_str()),
        ];
        assert!(
            !verify_webhook(PAYLOAD.as_bytes(), &headers, SECRET, at(NOW)),
            "{v2:?}"
        );
    }
}

#[test]
fn verify_webhook_falls_back_to_legacy_without_v2_header() {
    let legacy = make_sig(PAYLOAD.as_bytes(), SECRET);
    assert!(verify_webhook(
        PAYLOAD.as_bytes(),
        &[("x-wi-signature", legacy.as_str())],
        SECRET,
        VerifyOptions::default()
    ));
    let none: [(&str, &str); 0] = [];
    assert!(!verify_webhook(
        PAYLOAD.as_bytes(),
        &none,
        SECRET,
        VerifyOptions::default()
    ));
}

#[test]
fn from_headers_verifies_and_parses_or_returns_none() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let body = Bytes::from_static(PAYLOAD.as_bytes());
    let fresh = make_sig_v2(&body, SECRET, now);
    let ok = WebhookPayload::from_headers(
        body.clone(),
        &[("x-wi-signature-v2", fresh.as_str())],
        SECRET,
        VerifyOptions::default(),
    );
    assert_eq!(ok.unwrap().unwrap().event.event, "message");
    let stale = make_sig_v2(&body, SECRET, now - 3600);
    assert!(WebhookPayload::from_headers(
        body,
        &[("x-wi-signature-v2", stale.as_str())],
        SECRET,
        VerifyOptions::default()
    )
    .is_none());
}

// ─── receipt ──────────────────────────────────────────────────────────────────

fn receipt_body(
    raw_type: &str,
    status: Option<&str>,
    ids: &str,
    chat: &str,
    sender: &str,
) -> String {
    let status = status
        .map(|s| format!(r#""status":"{s}","#))
        .unwrap_or_default();
    format!(
        r#"{{"event":"receipt","session_id":"s","data":{{"type":"{raw_type}","rawType":"{raw_type}",{status}"chat":"{chat}","sender":"{sender}","messageIds":{ids},"timestamp":1748790003,"isFromMe":false,"messageFromMe":true}},"timestamp":"t"}}"#
    )
}

#[test]
fn receipt_payloads_decode_with_status_and_message_ids() {
    let dm = "5511998765432@s.whatsapp.net";
    let cases = [
        (
            "delivered",
            receipt_body("", Some("delivered"), r#"["A","B"]"#, dm, dm),
            Some(ReceiptStatus::Delivered),
            2,
        ),
        (
            "read",
            receipt_body("read", Some("read"), r#"["A"]"#, dm, dm),
            Some(ReceiptStatus::Read),
            1,
        ),
        (
            "played",
            receipt_body("played", Some("played"), r#"["C"]"#, dm, dm),
            Some(ReceiptStatus::Played),
            1,
        ),
        (
            "group",
            receipt_body(
                "read",
                Some("read"),
                r#"["G1"]"#,
                "120363025246125486@g.us",
                "5511912345678@s.whatsapp.net",
            ),
            Some(ReceiptStatus::Read),
            1,
        ),
        (
            "retry has no status",
            receipt_body("retry", None, r#"["A"]"#, dm, dm),
            None,
            1,
        ),
    ];
    for (name, body, status, ids) in cases {
        let event = parse_event(body.as_bytes()).unwrap();
        assert_eq!(event.event, "receipt", "{name}");
        let r: Receipt = serde_json::from_str(event.data.get()).unwrap();
        assert_eq!(r.status, status, "{name}");
        assert_eq!(r.message_ids.len(), ids, "{name}");
        assert_eq!(r.raw_type, r.receipt_type, "{name}");
        assert!(r.message_from_me, "{name}");
    }
}

// ─── disconnected ─────────────────────────────────────────────────────────────

use wi_api::types::{DisconnectReason, Disconnected};

fn disconnected_body(data: &str) -> String {
    format!(r#"{{"event":"disconnected","session_id":"s","data":{data},"timestamp":"t"}}"#)
}

#[test]
fn disconnected_payloads_decode_with_will_reconnect() {
    let cases: [(
        &str,
        &str,
        DisconnectReason,
        bool,
        Option<&str>,
        Option<i64>,
    ); 5] = [
        (
            "transient",
            r#"{"reason":"transient","willReconnect":true}"#,
            DisconnectReason::Transient,
            true,
            None,
            None,
        ),
        (
            "logged_out",
            r#"{"reason":"logged_out","willReconnect":false,"resume":"connect","code":401,"detail":"401: logged out"}"#,
            DisconnectReason::LoggedOut,
            false,
            Some("connect"),
            None,
        ),
        (
            "temporary_ban",
            r#"{"reason":"temporary_ban","willReconnect":false,"resume":"connect","code":101,"expireSeconds":3600,"retryAfterSeconds":3600}"#,
            DisconnectReason::TemporaryBan,
            false,
            Some("connect"),
            Some(3600),
        ),
        (
            "paused",
            r#"{"reason":"paused","willReconnect":false,"resume":"resume"}"#,
            DisconnectReason::Paused,
            false,
            Some("resume"),
            None,
        ),
        (
            "a reason a newer server adds",
            r#"{"reason":"brand_new","willReconnect":true}"#,
            DisconnectReason::Unknown("brand_new".into()),
            true,
            None,
            None,
        ),
    ];
    for (name, data, reason, will, resume, retry) in cases {
        let event = parse_event(disconnected_body(data).as_bytes()).unwrap();
        assert_eq!(event.event, "disconnected", "{name}");
        let d: Disconnected = serde_json::from_str(event.data.get())
            .unwrap_or_else(|e| panic!("{name}: an unknown reason must still decode: {e}"));
        assert_eq!(d.reason, reason, "{name}");
        assert_eq!(d.will_reconnect, will, "{name}");
        assert_eq!(d.resume.as_deref(), resume, "{name}");
        assert_eq!(d.retry_after_seconds, retry, "{name}");
    }
}

#[test]
fn disconnect_reason_known_set_is_closed() {
    let all = DisconnectReason::KNOWN;
    assert_eq!(
        all.iter().map(|r| r.as_str()).collect::<Vec<_>>(),
        [
            "transient",
            "manual",
            "paused",
            "logged_out",
            "replaced",
            "temporary_ban",
            "client_outdated",
            "connect_failure"
        ]
    );
    assert!(all.iter().all(|r| r.is_known()));
    assert!(!DisconnectReason::Unknown("brand_new".into()).is_known());
}

#[test]
fn disconnected_without_will_reconnect_is_rejected() {
    // The server always sends it; a payload without it is not a disconnected event.
    assert!(serde_json::from_str::<Disconnected>(r#"{"reason":"transient"}"#).is_err());
}

#[test]
fn parse_event_keeps_session_metadata() {
    let body = br#"{"event":"connected","session_id":"s","session_metadata":{"purpose":"briefing","team":"ops"},"data":{},"timestamp":"t"}"#;
    let event = parse_event(body).unwrap();
    assert_eq!(event.session_metadata.len(), 2);
    assert_eq!(event.session_metadata["purpose"], "briefing");
}

#[test]
fn parse_event_session_metadata_is_empty_when_absent_or_empty() {
    // An older server never sends it; a session without labels sends {}.
    let older = parse_event(PAYLOAD.as_bytes()).unwrap();
    assert!(older.session_metadata.is_empty());
    let none = br#"{"event":"connected","session_id":"s","session_metadata":{},"data":{},"timestamp":"t"}"#;
    assert!(parse_event(none).unwrap().session_metadata.is_empty());
}

// ─── pairing ──────────────────────────────────────────────────────────────────

use wi_api::types::{PairingFailed, PairingFailureReason, PairingMethod, PairingSucceeded};

fn pairing_body(event: &str, data: &str) -> String {
    format!(r#"{{"event":"{event}","session_id":"s","data":{data},"timestamp":"t"}}"#)
}

#[test]
fn pairing_failed_payloads_decode_with_method_reason_and_at() {
    let cases: [(&str, &str, PairingMethod, PairingFailureReason); 5] = [
        (
            "an unread QR",
            r#"{"method":"qr","reason":"timeout","at":"2026-11-13T12:02:40Z"}"#,
            PairingMethod::Qr,
            PairingFailureReason::Timeout,
        ),
        (
            "a code that expired",
            r#"{"method":"code","reason":"timeout","at":"2026-11-13T12:02:40Z"}"#,
            PairingMethod::Code,
            PairingFailureReason::Timeout,
        ),
        (
            "a refused link",
            r#"{"method":"qr","reason":"rejected","at":"2026-11-13T12:02:40Z"}"#,
            PairingMethod::Qr,
            PairingFailureReason::Rejected,
        ),
        (
            "a cancelled attempt",
            r#"{"method":"qr","reason":"cancelled","at":"2026-11-13T12:02:40Z"}"#,
            PairingMethod::Qr,
            PairingFailureReason::Cancelled,
        ),
        (
            "a reason a newer server adds",
            r#"{"method":"qr","reason":"brand_new","at":"2026-11-13T12:02:40Z"}"#,
            PairingMethod::Qr,
            PairingFailureReason::Unknown("brand_new".into()),
        ),
    ];
    for (name, data, method, reason) in cases {
        let event = parse_event(pairing_body("pairing.failed", data).as_bytes()).unwrap();
        assert_eq!(event.event, "pairing.failed", "{name}");
        let p: PairingFailed = serde_json::from_str(event.data.get())
            .unwrap_or_else(|e| panic!("{name}: an unknown reason must still decode: {e}"));
        assert_eq!(p.method, method, "{name}");
        assert_eq!(p.reason, reason, "{name}");
        assert_eq!(p.at, "2026-11-13T12:02:40Z", "{name}");
    }
}

#[test]
fn pairing_succeeded_decodes_and_an_unknown_method_does_not_fail() {
    let event = parse_event(
        pairing_body(
            "pairing.succeeded",
            r#"{"method":"code","at":"2026-11-13T12:02:40Z"}"#,
        )
        .as_bytes(),
    )
    .unwrap();
    let p: PairingSucceeded = serde_json::from_str(event.data.get()).unwrap();
    assert_eq!(p.method, PairingMethod::Code);
    let newer: PairingSucceeded = serde_json::from_str(r#"{"method":"passkey","at":"t"}"#).unwrap();
    assert_eq!(newer.method, PairingMethod::Unknown("passkey".into()));
    assert_eq!(newer.method.as_str(), "passkey");
}

#[test]
fn pairing_failure_reason_known_set_is_closed() {
    let all = PairingFailureReason::KNOWN;
    assert_eq!(
        all.iter().map(|r| r.as_str()).collect::<Vec<_>>(),
        [
            "timeout",
            "rejected",
            "cancelled",
            "client_outdated",
            "error"
        ]
    );
    assert!(all.iter().all(|r| r.is_known()));
    assert!(!PairingFailureReason::Unknown("brand_new".into()).is_known());
}
