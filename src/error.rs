use thiserror::Error;

#[derive(Debug, Error)]
pub enum WiError {
    #[error("HTTP {status}: {message}")]
    Api {
        status: u16,
        message: String,
        code: Option<String>,
    },

    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("serialize error: {0}")]
    Serialize(#[from] serde_json::Error),
}

impl WiError {
    pub(crate) async fn from_response(res: reqwest::Response) -> WiError {
        let status = res.status().as_u16();

        // Current shape: {"error":{"code","message"}}; the legacy flat shape
        // {"error":"...","message":"...","code":"..."} is still read.
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum ErrorField {
            Nested {
                code: Option<String>,
                message: Option<String>,
            },
            Flat(String),
        }

        #[derive(serde::Deserialize)]
        struct Body {
            message: Option<String>,
            error: Option<ErrorField>,
            code: Option<String>,
        }

        let fallback = format!("HTTP {status}");
        let (message, code) = match res.json::<Body>().await {
            Ok(Body {
                error: Some(ErrorField::Nested { code, message }),
                ..
            }) => (message.unwrap_or(fallback), code),
            Ok(b) => {
                let flat = match b.error {
                    Some(ErrorField::Flat(s)) => Some(s),
                    _ => None,
                };
                (b.message.or(flat).unwrap_or(fallback), b.code)
            }
            Err(_) => (fallback, None),
        };

        WiError::Api {
            status,
            message,
            code,
        }
    }
}
