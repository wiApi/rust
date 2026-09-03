use bytes::Bytes;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::{error::WiError, types::Event};

type HmacSha256 = Hmac<Sha256>;

const SIGNATURE_HEADER: &str = "x-wi-signature";

/// Verify the HMAC-SHA256 signature on a webhook payload.
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
    pub fn from_parts(body: Bytes, signature: &str, secret: &[u8]) -> Option<Result<Self, WiError>> {
        if !verify_signature(&body, signature, secret) {
            return None;
        }
        Some(parse_event(&body).map(|event| WebhookPayload { event, raw: body }))
    }

    /// Signature header name: `x-wi-signature`
    pub const HEADER: &'static str = SIGNATURE_HEADER;
}
