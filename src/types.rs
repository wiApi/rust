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
