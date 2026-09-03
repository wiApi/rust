use thiserror::Error;

#[derive(Debug, Error)]
pub enum WiError {
    #[error("HTTP {status}: {message}")]
    Api { status: u16, message: String, code: Option<String> },

    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("serialize error: {0}")]
    Serialize(#[from] serde_json::Error),
}

impl WiError {
    pub(crate) async fn from_response(res: reqwest::Response) -> WiError {
        let status = res.status().as_u16();

        #[derive(serde::Deserialize)]
        struct Body {
            message: Option<String>,
            error: Option<String>,
            code: Option<String>,
        }

        let message;
        let code;

        match res.json::<Body>().await {
            Ok(b) => {
                message = b.message.or(b.error).unwrap_or_else(|| format!("HTTP {status}"));
                code = b.code;
            }
            Err(_) => {
                message = format!("HTTP {status}");
                code = None;
            }
        }

        WiError::Api { status, message, code }
    }
}
