use crate::{error::WiError, types::*};
use std::sync::Arc;

/// A client scoped to a single WhatsApp session.
///
/// Obtain via [`crate::Wi::session`].
#[derive(Clone)]
pub struct Session {
    pub(crate) id: String,
    pub(crate) http: Arc<reqwest::Client>,
    pub(crate) base_url: Arc<String>,
    pub(crate) api_key: Arc<String>,
}

impl Session {
    fn url(&self, path: &str) -> String {
        format!("{}/sessions/{}{}", self.base_url, self.id, path)
    }

    async fn post<B: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<R, WiError> {
        let res = self
            .http
            .post(self.url(path))
            .header("x-api-key", self.api_key.as_str())
            .json(body)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(WiError::from_response(res).await);
        }
        Ok(res.json().await?)
    }

    async fn post_empty(&self, path: &str) -> Result<(), WiError> {
        let res = self
            .http
            .post(self.url(path))
            .header("x-api-key", self.api_key.as_str())
            .header("content-length", "0")
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(WiError::from_response(res).await);
        }
        Ok(())
    }

    async fn get<R: serde::de::DeserializeOwned>(&self, path: &str) -> Result<R, WiError> {
        let res = self
            .http
            .get(self.url(path))
            .header("x-api-key", self.api_key.as_str())
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(WiError::from_response(res).await);
        }
        Ok(res.json().await?)
    }

    // ─── Session lifecycle ────────────────────────────────────────────────────

    pub async fn status(&self) -> Result<SessionStatus, WiError> {
        self.get("/status").await
    }

    pub async fn qr(&self) -> Result<QRCode, WiError> {
        self.get("/qr").await
    }

    pub async fn connect(&self) -> Result<(), WiError> {
        self.post_empty("/connect").await
    }

    pub async fn disconnect(&self) -> Result<(), WiError> {
        self.post_empty("/disconnect").await
    }

    pub async fn logout(&self) -> Result<(), WiError> {
        self.post_empty("/logout").await
    }

    pub async fn pair_phone(&self, phone: &str) -> Result<PairPhoneResult, WiError> {
        #[derive(serde::Serialize)]
        struct Body<'a> { phone: &'a str }
        self.post("/pairphone", &Body { phone }).await
    }

    // ─── Send ─────────────────────────────────────────────────────────────────

    pub async fn send_text(&self, p: SendTextParams) -> Result<MessageResponse, WiError> {
        self.post("/messages/send-text", &p).await
    }

    pub async fn send_image(&self, p: SendImageParams) -> Result<MessageResponse, WiError> {
        self.post("/messages/send-image", &p).await
    }

    pub async fn send_audio(&self, p: SendAudioParams) -> Result<MessageResponse, WiError> {
        self.post("/messages/send-audio", &p).await
    }

    pub async fn send_video(&self, p: SendVideoParams) -> Result<MessageResponse, WiError> {
        self.post("/messages/send-video", &p).await
    }

    pub async fn send_document(&self, p: SendDocumentParams) -> Result<MessageResponse, WiError> {
        self.post("/messages/send-document", &p).await
    }

    pub async fn send_location(&self, p: SendLocationParams) -> Result<MessageResponse, WiError> {
        self.post("/messages/send-location", &p).await
    }

    pub async fn send_contact(&self, p: SendContactParams) -> Result<MessageResponse, WiError> {
        self.post("/messages/send-contact", &p).await
    }

    pub async fn send_sticker(&self, p: SendStickerParams) -> Result<MessageResponse, WiError> {
        self.post("/messages/send-sticker", &p).await
    }

    pub async fn send_poll(&self, p: SendPollParams) -> Result<MessageResponse, WiError> {
        self.post("/messages/send-poll", &p).await
    }

    pub async fn react(&self, p: SendReactionParams) -> Result<(), WiError> {
        self.post::<_, serde_json::Value>("/chat/react", &p).await?;
        Ok(())
    }

    // ─── Chat ─────────────────────────────────────────────────────────────────

    pub async fn mark_read(&self, p: MarkReadParams) -> Result<(), WiError> {
        self.post::<_, serde_json::Value>("/chat/markread", &p).await?;
        Ok(())
    }

    pub async fn presence(&self, p: PresenceParams) -> Result<(), WiError> {
        self.post::<_, serde_json::Value>("/chat/presence", &p).await?;
        Ok(())
    }
}
