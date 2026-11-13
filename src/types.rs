use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

// ─── Session ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct SessionStatus {
    pub connected: bool,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QRCode {
    pub qr: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PairPhoneResult {
    pub pair_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
    pub id: String,
}

// ─── Send params ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Default)]
pub struct SendTextParams {
    pub to: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quoted: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SendImageParams {
    pub to: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quoted: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SendAudioParams {
    pub to: String,
    pub url: String,
    /// Send as voice note. Default: false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ptt: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SendVideoParams {
    pub to: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quoted: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SendDocumentParams {
    pub to: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SendLocationParams {
    pub to: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SendContactParams {
    pub to: String,
    pub contact_name: String,
    pub contact_number: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SendStickerParams {
    pub to: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SendPollParams {
    pub to: String,
    pub question: String,
    pub options: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_answers: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SendReactionParams {
    pub to: String,
    pub message_id: String,
    /// Emoji character, or empty string to remove the reaction.
    pub emoji: String,
}

// ─── Chat params ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Default)]
pub struct MarkReadParams {
    pub chat_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct PresenceParams {
    pub chat_id: String,
    /// One of "composing", "recording", "paused"
    pub presence: String,
}

// ─── Webhooks ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct Event {
    pub event: String,
    pub session_id: String,
    /// The session's `metadata` (labels set on `POST /sessions` or `PATCH /sessions/{id}`) when
    /// the event was sent, so you can route on it without a lookup. Empty when the session has
    /// none, and also empty on a server older than this field.
    #[serde(default)]
    pub session_metadata: std::collections::HashMap<String, String>,
    pub data: Box<RawValue>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncomingMessage {
    pub id: String,
    pub chat: String,
    pub from: String,
    pub from_me: bool,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub text: Option<String>,
    pub caption: Option<String>,
    pub media_url: Option<String>,
    pub mimetype: Option<String>,
    pub filename: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timestamp: i64,
}

/// Why a session went offline. The set is closed, but a newer server may add a
/// value: it decodes as [`DisconnectReason::Unknown`] instead of failing, so
/// decide by [`Disconnected::will_reconnect`] when [`is_known`](Self::is_known)
/// is false.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisconnectReason {
    /// Network or WhatsApp drop the engine retries by itself.
    Transient,
    /// A disconnect you asked for.
    Manual,
    /// A durable pause you asked for; resume it explicitly.
    Paused,
    /// The link was removed; pair again.
    LoggedOut,
    /// Another client took over the session.
    Replaced,
    /// Temporary ban; the engine reconnects after `retry_after_seconds`.
    TemporaryBan,
    /// WhatsApp refused the engine's client version.
    ClientOutdated,
    /// A connect error WhatsApp reported (see `code`).
    ConnectFailure,
    /// A value this SDK version does not know.
    Unknown(String),
}

impl DisconnectReason {
    /// Every reason the server publishes today.
    pub const KNOWN: [DisconnectReason; 8] = [
        DisconnectReason::Transient,
        DisconnectReason::Manual,
        DisconnectReason::Paused,
        DisconnectReason::LoggedOut,
        DisconnectReason::Replaced,
        DisconnectReason::TemporaryBan,
        DisconnectReason::ClientOutdated,
        DisconnectReason::ConnectFailure,
    ];

    /// The wire value.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Transient => "transient",
            Self::Manual => "manual",
            Self::Paused => "paused",
            Self::LoggedOut => "logged_out",
            Self::Replaced => "replaced",
            Self::TemporaryBan => "temporary_ban",
            Self::ClientOutdated => "client_outdated",
            Self::ConnectFailure => "connect_failure",
            Self::Unknown(other) => other,
        }
    }

    /// False for a reason a newer server added.
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Unknown(_))
    }
}

impl<'de> Deserialize<'de> for DisconnectReason {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        Ok(match raw.as_str() {
            "transient" => Self::Transient,
            "manual" => Self::Manual,
            "paused" => Self::Paused,
            "logged_out" => Self::LoggedOut,
            "replaced" => Self::Replaced,
            "temporary_ban" => Self::TemporaryBan,
            "client_outdated" => Self::ClientOutdated,
            "connect_failure" => Self::ConnectFailure,
            _ => Self::Unknown(raw),
        })
    }
}

/// Data payload of the `disconnected` event. `will_reconnect` is always
/// present: `true` means the engine reconnects by itself, `false` means the
/// session stays offline until you act (see `resume`).
/// Decode with `serde_json::from_str::<Disconnected>(event.data.get())`.
#[derive(Debug, Clone, Deserialize)]
pub struct Disconnected {
    pub reason: DisconnectReason,
    #[serde(rename = "willReconnect")]
    pub will_reconnect: bool,
    /// Call that brings the session back (`connect` or `resume`). Only with
    /// `will_reconnect: false`.
    pub resume: Option<String>,
    /// Code WhatsApp reported (`logged_out`, `temporary_ban`, `connect_failure`).
    pub code: Option<i64>,
    /// Diagnostic text; do not branch on it. `"local"` marks a logout you asked for.
    pub detail: Option<String>,
    /// `temporary_ban`: length of the ban, in seconds.
    #[serde(rename = "expireSeconds")]
    pub expire_seconds: Option<i64>,
    /// `temporary_ban`: seconds until the engine reconnects by itself.
    #[serde(rename = "retryAfterSeconds")]
    pub retry_after_seconds: Option<i64>,
}

/// How a pairing attempt links the device. A value a newer server adds decodes
/// as [`PairingMethod::Unknown`] instead of failing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairingMethod {
    /// A scan of the QR code.
    Qr,
    /// A pairing code typed on the phone (`pair_phone`).
    Code,
    /// A value this SDK version does not know.
    Unknown(String),
}

impl PairingMethod {
    /// The wire value.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Qr => "qr",
            Self::Code => "code",
            Self::Unknown(other) => other,
        }
    }
}

impl<'de> Deserialize<'de> for PairingMethod {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        Ok(match raw.as_str() {
            "qr" => Self::Qr,
            "code" => Self::Code,
            _ => Self::Unknown(raw),
        })
    }
}

/// Why a pairing attempt ended without linking the number. The set is closed,
/// but a newer server may add a value: it decodes as
/// [`PairingFailureReason::Unknown`] instead of failing, so treat it like
/// [`PairingFailureReason::Error`] when [`is_known`](Self::is_known) is false.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairingFailureReason {
    /// Nobody read the QR or typed the code in time (the QR channel lasts about 160 s).
    Timeout,
    /// WhatsApp refused the link.
    Rejected,
    /// `disconnect` or `pause` ended the attempt.
    Cancelled,
    /// WhatsApp refuses the connector's version.
    ClientOutdated,
    /// Any other failure.
    Error,
    /// A value this SDK version does not know.
    Unknown(String),
}

impl PairingFailureReason {
    /// Every reason the server publishes today.
    pub const KNOWN: [PairingFailureReason; 5] = [
        PairingFailureReason::Timeout,
        PairingFailureReason::Rejected,
        PairingFailureReason::Cancelled,
        PairingFailureReason::ClientOutdated,
        PairingFailureReason::Error,
    ];

    /// The wire value.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Timeout => "timeout",
            Self::Rejected => "rejected",
            Self::Cancelled => "cancelled",
            Self::ClientOutdated => "client_outdated",
            Self::Error => "error",
            Self::Unknown(other) => other,
        }
    }

    /// False for a reason a newer server added.
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Unknown(_))
    }
}

impl<'de> Deserialize<'de> for PairingFailureReason {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        Ok(match raw.as_str() {
            "timeout" => Self::Timeout,
            "rejected" => Self::Rejected,
            "cancelled" => Self::Cancelled,
            "client_outdated" => Self::ClientOutdated,
            "error" => Self::Error,
            _ => Self::Unknown(raw),
        })
    }
}

/// Data payload of the `pairing.failed` event: how a pairing attempt ended
/// without linking the number. `at` is the end of the attempt, ISO 8601.
/// Decode with `serde_json::from_str::<PairingFailed>(event.data.get())`.
#[derive(Debug, Clone, Deserialize)]
pub struct PairingFailed {
    pub method: PairingMethod,
    pub reason: PairingFailureReason,
    pub at: String,
}

/// Data payload of the `pairing.succeeded` event, sent just before `connected`.
/// Decode with `serde_json::from_str::<PairingSucceeded>(event.data.get())`.
#[derive(Debug, Clone, Deserialize)]
pub struct PairingSucceeded {
    pub method: PairingMethod,
    pub at: String,
}

/// Normalized delivery state of a [`Receipt`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReceiptStatus {
    Delivered,
    Read,
    Played,
}

/// Data payload of the `receipt` event. One receipt can acknowledge several
/// messages (`message_ids`); in groups there is one receipt per participant.
/// Decode with `serde_json::from_str::<Receipt>(event.data.get())`.
#[derive(Debug, Clone, Deserialize)]
pub struct Receipt {
    /// `None` for receipt types that say nothing about delivery
    /// (`retry`, `server-error`, `inactive`, `peer_msg`, `hist_sync`).
    pub status: Option<ReceiptStatus>,
    /// Raw WhatsApp type: `""` (delivered), `sender`, `read`, `read-self`,
    /// `played`, `played-self`, ... `type` carries the same value.
    #[serde(rename = "rawType")]
    pub raw_type: String,
    #[serde(rename = "type")]
    pub receipt_type: String,
    pub chat: String,
    /// Who produced the receipt: the contact, or a group participant.
    pub sender: String,
    #[serde(rename = "messageIds")]
    pub message_ids: Vec<String>,
    /// Unix seconds.
    pub timestamp: i64,
    /// The receipt came from another device of your own account.
    #[serde(rename = "isFromMe")]
    pub is_from_me: bool,
    /// The acknowledged messages were sent by you: only these update the
    /// status of sent messages.
    #[serde(rename = "messageFromMe")]
    pub message_from_me: bool,
    /// Author of the acknowledged messages when it is not you (group reads).
    #[serde(rename = "messageSender")]
    pub message_sender: Option<String>,
}
