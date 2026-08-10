//! Official [Anti-Captcha.com](https://anti-captcha.com) API client.
//!
//! Solves image captchas, Recaptcha v2/v3 (Enterprise and non-Enterprise),
//! hCaptcha, FunCaptcha (Arkose Labs), GeeTest v3/v4, Cloudflare Turnstile,
//! Amazon WAF, Prosopo, Friendly Captcha, Altcha, and runs AntiGate custom
//! scenarios.
//!
//! To use the service you need to [register](https://anti-captcha.com/clients/)
//! and top up your balance. For the API details see the
//! [documentation](https://anti-captcha.com/apidoc).
//!
//! # Example
//!
//! ```no_run
//! use anticaptcha::{Client, RecaptchaV2};
//!
//! # async fn run() -> anticaptcha::Result<()> {
//! let ac = Client::new("API_KEY_HERE");
//!
//! // Make sure the API key funds balance is positive
//! println!("Balance: {}", ac.get_balance().await?);
//!
//! let solution = ac
//!     .solve_recaptcha_v2(&RecaptchaV2 {
//!         website_url: "https://www.website.com/".into(),
//!         website_key: "6Lcyu8UZAAAAACwSh6Xf58WrNXTu0LLu4F85xf20".into(),
//!         ..Default::default()
//!     })
//!     .await?;
//!
//! println!("g-response token: {}", solution.g_recaptcha_response());
//! # Ok(())
//! # }
//! ```
//!
//! # Synchronous API
//!
//! Enable the `blocking` feature to get [`blocking::Client`], a synchronous
//! mirror of the same methods:
//!
//! ```toml
//! anticaptchaofficial = { version = "1", features = ["blocking"] }
//! ```
//!
//! # Feature flags
//!
//! - `rustls-tls` *(default)* — TLS through rustls, no system OpenSSL needed
//! - `native-tls` — TLS through the platform library
//! - `blocking` — adds the synchronous [`blocking`] module

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

#[cfg(feature = "blocking")]
pub mod blocking;

mod client;
mod error;
mod solution;
mod tasks;

/// Re-exported so you can build `variables` and `init_parameters` without
/// adding `serde_json` to your own `Cargo.toml`.
pub use serde_json;

pub use client::Client;
pub use error::{Error, Result};
pub use solution::Solution;
pub use tasks::{
    Altcha, Amazon, AntiBotCookie, AntiGate, FriendlyCaptcha, FunCaptcha, GeeTest, HCaptcha,
    ImageSettings, ImageToCoordinates, Prosopo, Proxy, ProxyType, RecaptchaV2, RecaptchaV3,
    Turnstile,
};

/// Library version, e.g. `"1.0.0"`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
