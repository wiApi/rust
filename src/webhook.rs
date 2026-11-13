use bytes::Bytes;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::{error::WiError, types::Event};

type HmacSha256 = Hmac<Sha256>;

const SIGNATURE_HEADER: &str = "x-wi-signature";
const SIGNATURE_V2_HEADER: &str = "x-wi-signature-v2";

/// Default max age of the signed timestamp (`x-wi-timestamp`), in seconds: 5 minutes.
pub const DEFAULT_TOLERANCE_SECONDS: u64 = 300;

/// Options for [`verify_webhook`].
#[derive(Debug, Clone, Copy)]
pub struct VerifyOptions {
    /// Max distance between the signed timestamp and now, in seconds.
    pub tolerance_seconds: u64,
    /// Current unix time in seconds; `None` reads the system clock. For tests.
    pub now: Option<i64>,
}

impl Default for VerifyOptions {
    fn default() -> Self {
        Self {
            tolerance_seconds: DEFAULT_TOLERANCE_SECONDS,
            now: None,
        }
    }
}

fn header_value<'a, K: AsRef<str>, V: AsRef<str>>(
    headers: &'a [(K, V)],
    name: &str,
) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k.as_ref().eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_ref())
}

/// Verify a webhook delivery with replay protection.
///
/// Checks `x-wi-signature-v2: t=<unix seconds>,v1=<hex>`, the HMAC-SHA256 of
/// `"<t>.<body>"`, and rejects it when `t` is further than
/// `options.tolerance_seconds` from now: a captured request cannot be replayed
/// later, and a tampered timestamp fails the signature. A delivery with no v2
/// header (older servers) falls back to the legacy `x-wi-signature` body
/// signature, which has no replay protection. A malformed v2 header is
/// rejected, never downgraded.
///
/// `headers` is any list of `(name, value)` pairs; names match case-insensitively.
pub fn verify_webhook<K: AsRef<str>, V: AsRef<str>>(
    body: &[u8],
    headers: &[(K, V)],
    secret: &[u8],
    options: VerifyOptions,
) -> bool {
    let Some(v2) = header_value(headers, SIGNATURE_V2_HEADER) else {
        return header_value(headers, SIGNATURE_HEADER)
            .is_some_and(|legacy| verify_signature(body, legacy, secret));
    };

    let (mut t, mut v1) = (None, None);
    for part in v2.split(',') {
        match part.trim().split_once('=') {
            Some(("t", v)) => t = Some(v),
            Some(("v1", v)) => v1 = Some(v),
            _ => {}
        }
    }
    let (Some(t_str), Some(v1)) = (t, v1) else {
        return false;
    };
    let Ok(t_val) = t_str.parse::<i64>() else {
        return false;
    };
    let Ok(expected) = hex::decode(v1) else {
        return false;
    };

    let now = options.now.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    });
    if now.abs_diff(t_val) > options.tolerance_seconds {
        return false;
    }

    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(t_str.as_bytes());
    mac.update(b".");
    mac.update(body);
    mac.verify_slice(&expected).is_ok()
}

/// Verify the legacy body-only HMAC-SHA256 signature on a webhook payload
/// (no replay protection). Prefer [`verify_webhook`].
///
/// The server sends `x-wi-signature: sha256=<hex>`. Pass the raw body bytes,
/// the header value, and your webhook secret.
///
/// Returns `false` (not an error) when the signature does not match — treat
/// that as an HTTP 401.
pub fn verify_signature(body: &[u8], signature: &str, secret: &[u8]) -> bool {
    let hex_part = match signature.strip_prefix("sha256=") {
        Some(h) => h,
        None => return false,
    };

    let expected = match hex::decode(hex_part) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(body);
    mac.verify_slice(&expected).is_ok()
}

/// Parse raw webhook body bytes into an [`Event`].
pub fn parse_event(body: &[u8]) -> Result<Event, WiError> {
    serde_json::from_slice(body).map_err(WiError::Serialize)
}

/// A verified, parsed webhook request.
pub struct WebhookPayload {
    pub event: Event,
    pub raw: Bytes,
}

impl WebhookPayload {
    /// Verify the signature and parse the event from raw bytes and a header value.
    ///
    /// Returns `None` when the signature is invalid (caller should respond 401).
    pub fn from_parts(
        body: Bytes,
        signature: &str,
        secret: &[u8],
    ) -> Option<Result<Self, WiError>> {
        if !verify_signature(&body, signature, secret) {
            return None;
        }
        Some(parse_event(&body).map(|event| WebhookPayload { event, raw: body }))
    }

    /// Verify with replay protection ([`verify_webhook`]) and parse the event.
    ///
    /// Returns `None` when verification fails (caller should respond 401).
    pub fn from_headers<K: AsRef<str>, V: AsRef<str>>(
        body: Bytes,
        headers: &[(K, V)],
        secret: &[u8],
        options: VerifyOptions,
    ) -> Option<Result<Self, WiError>> {
        if !verify_webhook(&body, headers, secret, options) {
            return None;
        }
        Some(parse_event(&body).map(|event| WebhookPayload { event, raw: body }))
    }

    /// Signature header name: `x-wi-signature`
    pub const HEADER: &'static str = SIGNATURE_HEADER;
}
