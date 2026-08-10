//! Synchronous mirror of the async API, behind the `blocking` feature.
//!
//! Every method has the same name and takes the same arguments as its
//! [`crate::Client`] counterpart, it just blocks instead of returning a future.
//!
//! ```no_run
//! use anticaptcha::blocking::Client;
//! use anticaptcha::RecaptchaV2;
//!
//! # fn run() -> anticaptcha::Result<()> {
//! let ac = Client::new("API_KEY_HERE")?;
//!
//! let solution = ac.solve_recaptcha_v2(&RecaptchaV2 {
//!     website_url: "https://www.website.com/".into(),
//!     website_key: "SITE_KEY".into(),
//!     ..Default::default()
//! })?;
//!
//! println!("{}", solution.g_recaptcha_response());
//! # Ok(())
//! # }
//! ```
//!
//! # Do not call this from inside an async runtime
//!
//! The blocking client drives its own Tokio runtime, and nesting runtimes
//! panics. If your program is already async, use [`crate::Client`] directly.

use std::path::Path;
use std::time::Duration;

use serde_json::Value;

use crate::error::{Error, Result};
use crate::solution::Solution;
use crate::tasks;

/// Forwards a set of async methods to their blocking counterparts.
macro_rules! forward {
    ($( $(#[$meta:meta])* $name:ident ( $($arg:ident : $ty:ty),* ) -> $ret:ty; )*) => {
        $(
            $(#[$meta])*
            pub fn $name(&self, $($arg: $ty),*) -> Result<$ret> {
                self.runtime.block_on(self.inner.$name($($arg),*))
            }
        )*
    };
}

/// Synchronous Anti-Captcha client.
#[derive(Debug)]
pub struct Client {
    inner: crate::Client,
    runtime: tokio::runtime::Runtime,
}

impl Client {
    /// Creates a client and the runtime that drives it.
    ///
    /// Fails only when the Tokio runtime cannot be started.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::from_async(crate::Client::new(api_key))
    }

    /// Wraps an already configured async client.
    pub fn from_async(inner: crate::Client) -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|source| Error::Io {
                path: "tokio runtime".into(),
                source,
            })?;

        Ok(Self { inner, runtime })
    }

    /// The async client underneath, for the settings builders.
    pub fn inner(&self) -> &crate::Client {
        &self.inner
    }

    pub fn set_api_key(&mut self, api_key: impl Into<String>) {
        self.inner.set_api_key(api_key);
    }

    /// Applies a builder method of the async client, e.g.
    /// `client.configure(|c| c.with_soft_id(1187).quiet())`.
    pub fn configure(&mut self, change: impl FnOnce(crate::Client) -> crate::Client) {
        let current = self.inner.clone();
        self.inner = change(current);
    }

    /// Reads a file and encodes it in base64.
    pub fn file_to_base64(path: impl AsRef<Path>) -> Result<String> {
        crate::Client::file_to_base64(path)
    }

    /// Encodes bytes in base64.
    pub fn to_base64(bytes: impl AsRef<[u8]>) -> String {
        crate::Client::to_base64(bytes)
    }

    forward! {
        /// Account balance in US dollars.
        get_balance() -> f64;
        /// Number of prepaid captcha credits on the account.
        get_credits_balance() -> f64;

        /// Calls any API method with your own payload.
        json_request(method: &str, payload: Value) -> Value;
        /// Submits a task and returns its id.
        create_task(task: Value) -> i64;
        /// Polls until the task is solved or the waiting limit runs out.
        wait_for_result(task_id: i64) -> Solution;
        /// Sends a task type this library does not wrap yet.
        solve_task(task: Value) -> Solution;

        solve_image(base64_body: &str, settings: &tasks::ImageSettings) -> Solution;
        solve_image_to_coordinates(base64_body: &str, settings: &tasks::ImageToCoordinates) -> Solution;
        solve_recaptcha_v2(settings: &tasks::RecaptchaV2) -> Solution;
        solve_recaptcha_v2_proxy_on(settings: &tasks::RecaptchaV2) -> Solution;
        solve_recaptcha_v3(settings: &tasks::RecaptchaV3) -> Solution;
        solve_hcaptcha(settings: &tasks::HCaptcha) -> Solution;
        solve_hcaptcha_proxy_on(settings: &tasks::HCaptcha) -> Solution;
        solve_funcaptcha(settings: &tasks::FunCaptcha) -> Solution;
        solve_funcaptcha_proxy_on(settings: &tasks::FunCaptcha) -> Solution;
        solve_geetest(settings: &tasks::GeeTest) -> Solution;
        solve_geetest_proxy_on(settings: &tasks::GeeTest) -> Solution;
        solve_turnstile(settings: &tasks::Turnstile) -> Solution;
        solve_turnstile_proxy_on(settings: &tasks::Turnstile) -> Solution;
        solve_prosopo(settings: &tasks::Prosopo) -> Solution;
        solve_prosopo_proxy_on(settings: &tasks::Prosopo) -> Solution;
        solve_friendly_captcha(settings: &tasks::FriendlyCaptcha) -> Solution;
        solve_friendly_captcha_proxy_on(settings: &tasks::FriendlyCaptcha) -> Solution;
        solve_altcha(settings: &tasks::Altcha) -> Solution;
        solve_altcha_proxy_on(settings: &tasks::Altcha) -> Solution;
        solve_amazon(settings: &tasks::Amazon) -> Solution;
        solve_amazon_proxy_on(settings: &tasks::Amazon) -> Solution;
        solve_antigate(settings: &tasks::AntiGate) -> Solution;
        solve_antibot_cookie(settings: &tasks::AntiBotCookie) -> Solution;

        report_incorrect_image_captcha(task_id: i64) -> ();
        report_incorrect_recaptcha(task_id: i64) -> ();
        report_correct_recaptcha(task_id: i64) -> ();
        report_incorrect_hcaptcha(task_id: i64) -> ();
    }

    /// Solves an image captcha read from a file.
    pub fn solve_image_file(
        &self,
        path: impl AsRef<Path>,
        settings: &tasks::ImageSettings,
    ) -> Result<Solution> {
        self.runtime
            .block_on(self.inner.solve_image_file(path, settings))
    }

    /// Solves an image-to-coordinates captcha read from a file.
    pub fn solve_image_to_coordinates_file(
        &self,
        path: impl AsRef<Path>,
        settings: &tasks::ImageToCoordinates,
    ) -> Result<Solution> {
        self.runtime
            .block_on(self.inner.solve_image_to_coordinates_file(path, settings))
    }

    /// How long to keep polling before giving up. Default 300 seconds.
    pub fn set_max_waiting_time(&mut self, timeout: Duration) {
        self.configure(|client| client.with_max_waiting_time(timeout));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_api_key_is_refused() {
        let client = Client::from_async(crate::Client::new("").quiet()).unwrap();
        let error = client.get_balance().unwrap_err();

        assert!(matches!(error, Error::InvalidTask(_)));
    }

    #[test]
    fn configure_replaces_the_inner_client() {
        let mut client = Client::from_async(crate::Client::new("").quiet()).unwrap();
        client.set_max_waiting_time(Duration::from_secs(42));
        client.configure(|inner| inner.with_soft_id(1187));
        client.set_api_key("KEY");

        // The settings are private, so just check the client is still usable and
        // still validates before touching the network.
        let error = client.solve_altcha(&crate::Altcha::default()).unwrap_err();

        assert!(matches!(error, Error::InvalidTask(_)));
    }
}
