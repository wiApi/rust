//! Official Rust SDK for the [wi-api](https://wi.api.br) WhatsApp platform.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use wi_api::{Wi, types::SendTextParams};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let wi = Wi::new("YOUR_API_KEY");
//!     let session = wi.session("my-instance");
//!
//!     let msg = session.send_text(SendTextParams {
//!         to: "5511999999999".into(),
//!         text: "Hello from wi-api".into(),
//!         ..Default::default()
//!     }).await?;
//!
//!     println!("sent: {}", msg.message_id);
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod session;
pub mod types;
pub mod webhook;

pub use error::WiError;
pub use session::Session;

use std::sync::Arc;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://endpoint.wi.api.br";

/// The wi-api client. Create with [`Wi::new`] or [`Wi::builder`].
#[derive(Clone)]
pub struct Wi {
    http: Arc<reqwest::Client>,
    base_url: Arc<String>,
    api_key: Arc<String>,
}

impl Wi {
    /// Create a client with the given API key and default settings.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::builder(api_key).build()
    }

    /// Create a [`WiBuilder`] for custom configuration.
    pub fn builder(api_key: impl Into<String>) -> WiBuilder {
        WiBuilder {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Return a [`Session`] scoped to the given session ID.
    pub fn session(&self, id: impl Into<String>) -> Session {
        Session {
            id: id.into(),
            http: self.http.clone(),
            base_url: self.base_url.clone(),
            api_key: self.api_key.clone(),
        }
    }
}

/// Builder for [`Wi`].
pub struct WiBuilder {
    api_key: String,
    base_url: String,
    timeout: Duration,
}

impl WiBuilder {
    /// Override the API base URL. Default: `https://endpoint.wi.api.br`.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into().trim_end_matches('/').to_string();
        self
    }

    /// Set the per-request timeout. Default: 30s.
    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = d;
        self
    }

    pub fn build(self) -> Wi {
        let http = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .expect("failed to build HTTP client");

        Wi {
            http: Arc::new(http),
            base_url: Arc::new(self.base_url),
            api_key: Arc::new(self.api_key),
        }
    }
}
