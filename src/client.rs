//! The async API client.

use std::time::{Duration, Instant};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use serde_json::{json, Value};

use crate::error::{Error, Result};
use crate::solution::Solution;
use crate::tasks;

const API_HOST: &str = "https://api.anti-captcha.com/";

/// Talks to the Anti-Captcha API.
///
/// The client holds no per-task state, so one instance can solve any number of
/// captchas concurrently. Cloning is cheap: the underlying connection pool is
/// shared.
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    api_key: String,
    soft_id: i64,
    verbose: bool,
    connection_timeout: Duration,
    first_attempt_waiting_interval: Duration,
    normal_waiting_interval: Duration,
    max_waiting_time: Duration,
}

impl Client {
    /// Creates a client for the given key from
    /// <https://anti-captcha.com/clients/settings/apisetup>.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key: api_key.into(),
            soft_id: 0,
            verbose: true,
            connection_timeout: Duration::from_secs(30),
            first_attempt_waiting_interval: Duration::from_secs(5),
            normal_waiting_interval: Duration::from_secs(5),
            max_waiting_time: Duration::from_secs(300),
        }
    }

    /// Reuses an existing `reqwest` client, so the connection pool, proxy and
    /// TLS settings are shared with the rest of your program.
    pub fn with_http_client(mut self, http: reqwest::Client) -> Self {
        self.http = http;
        self
    }

    pub fn set_api_key(&mut self, api_key: impl Into<String>) {
        self.api_key = api_key.into();
    }

    /// Specify a soft id to earn 10% commission with your app.
    /// Get yours at <https://anti-captcha.com/clients/tools/devcenter>.
    pub fn with_soft_id(mut self, soft_id: i64) -> Self {
        self.soft_id = soft_id;
        self
    }

    /// Debug output goes to stderr. On by default.
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Same as `with_verbose(false)`.
    pub fn quiet(self) -> Self {
        self.with_verbose(false)
    }

    /// Timeout of a single HTTP call to the API. Default 30 seconds.
    pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// How long to wait before the first status request, and between the
    /// following ones. Default 5 seconds each.
    pub fn with_polling_intervals(mut self, first: Duration, normal: Duration) -> Self {
        self.first_attempt_waiting_interval = first;
        self.normal_waiting_interval = normal;
        self
    }

    /// How long to keep polling before giving up. Default 300 seconds.
    pub fn with_max_waiting_time(mut self, timeout: Duration) -> Self {
        self.max_waiting_time = timeout;
        self
    }

    fn log(&self, message: impl std::fmt::Display) {
        if self.verbose {
            eprintln!("[anticaptcha] {message}");
        }
    }

    // ------------------------------------------------------------------ money

    /// Account balance in US dollars.
    pub async fn get_balance(&self) -> Result<f64> {
        let response = self.json_request("getBalance", json!({})).await?;

        Ok(response
            .get("balance")
            .and_then(Value::as_f64)
            .unwrap_or_default())
    }

    /// Number of prepaid captcha credits on the account.
    pub async fn get_credits_balance(&self) -> Result<f64> {
        let response = self.json_request("getBalance", json!({})).await?;

        Ok(response
            .get("captchaCredits")
            .and_then(Value::as_f64)
            .unwrap_or_default())
    }

    // ------------------------------------------------------------- low level

    /// Calls any API method with your own payload. The client key is added for you.
    pub async fn json_request(&self, method: &str, mut payload: Value) -> Result<Value> {
        if self.api_key.is_empty() {
            return Err(Error::InvalidTask("API key is not set".into()));
        }

        if let Value::Object(fields) = &mut payload {
            fields.insert("clientKey".into(), json!(self.api_key));
        }

        let url = format!("{API_HOST}{method}");
        self.log(format_args!("POST {url}"));

        let response = self
            .http
            .post(&url)
            .timeout(self.connection_timeout)
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        let raw = response.text().await?;

        let body: Value = serde_json::from_str(&raw).map_err(|error| Error::BadResponse {
            message: error.to_string(),
            raw: raw.chars().take(500).collect(),
        })?;

        if !status.is_success() {
            return Err(Error::BadResponse {
                message: format!("API answered with HTTP {status}"),
                raw: raw.chars().take(500).collect(),
            });
        }

        let error_id = body.get("errorId").and_then(Value::as_i64).unwrap_or(-1);

        if error_id != 0 {
            return Err(Error::Api {
                error_id,
                error_code: body
                    .get("errorCode")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                description: body
                    .get("errorDescription")
                    .and_then(Value::as_str)
                    .unwrap_or("(no error description)")
                    .to_owned(),
            });
        }

        Ok(body)
    }

    /// Submits a task and returns its id.
    pub async fn create_task(&self, task: Value) -> Result<i64> {
        self.log(format_args!("Creating task: {task}"));

        let response = self
            .json_request(
                "createTask",
                json!({ "task": task, "softId": self.soft_id }),
            )
            .await?;

        let task_id = response
            .get("taskId")
            .and_then(Value::as_i64)
            .ok_or_else(|| Error::BadResponse {
                message: "API did not return a task id".into(),
                raw: response.to_string().chars().take(500).collect(),
            })?;

        self.log(format_args!("Task id: {task_id}"));

        Ok(task_id)
    }

    /// Polls until the task is solved or the waiting limit runs out.
    pub async fn wait_for_result(&self, task_id: i64) -> Result<Solution> {
        let deadline = Instant::now() + self.max_waiting_time;
        let mut interval = self.first_attempt_waiting_interval;

        loop {
            self.log(format_args!("Waiting {} seconds...", interval.as_secs()));
            tokio::time::sleep(interval).await;
            interval = self.normal_waiting_interval;

            let response = self
                .json_request("getTaskResult", json!({ "taskId": task_id }))
                .await?;

            let status = response
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or_default();

            match status {
                "ready" => {
                    let cost = response
                        .get("cost")
                        .and_then(Value::as_f64)
                        .unwrap_or_default();

                    let solution = response
                        .get("solution")
                        .filter(|value| value.is_object())
                        .ok_or_else(|| Error::BadResponse {
                            message: "the task is ready but carries no solution".into(),
                            raw: response.to_string().chars().take(500).collect(),
                        })?;

                    let solution = Solution::new(solution.clone());

                    if solution.is_empty() {
                        return Err(Error::BadResponse {
                            message: "the API returned an empty solution".into(),
                            raw: response.to_string().chars().take(500).collect(),
                        });
                    }

                    self.log("The task is complete");

                    return Ok(solution.with_task(task_id, cost));
                }
                "processing" => {
                    if Instant::now() >= deadline {
                        return Err(Error::Timeout(self.max_waiting_time.as_secs()));
                    }

                    self.log("The task is still processing...");
                }
                other => {
                    return Err(Error::BadResponse {
                        message: format!(
                            "unknown task status \"{other}\", please update the library"
                        ),
                        raw: response.to_string().chars().take(500).collect(),
                    })
                }
            }
        }
    }

    /// Sends a task type this library does not wrap yet, and waits for the result.
    /// See <https://anti-captcha.com/apidoc/task-types> for the `task` object shape.
    pub async fn solve_task(&self, task: Value) -> Result<Solution> {
        let task_id = self.create_task(task).await?;

        self.wait_for_result(task_id).await
    }

    // ----------------------------------------------------------------- tasks

    pub async fn solve_image(
        &self,
        base64_body: &str,
        settings: &tasks::ImageSettings,
    ) -> Result<Solution> {
        self.solve_task(tasks::build_image(base64_body, settings)?)
            .await
    }

    pub async fn solve_image_file(
        &self,
        path: impl AsRef<std::path::Path>,
        settings: &tasks::ImageSettings,
    ) -> Result<Solution> {
        self.solve_image(&Self::file_to_base64(path)?, settings)
            .await
    }

    pub async fn solve_image_to_coordinates(
        &self,
        base64_body: &str,
        settings: &tasks::ImageToCoordinates,
    ) -> Result<Solution> {
        self.solve_task(tasks::build_image_to_coordinates(base64_body, settings)?)
            .await
    }

    pub async fn solve_image_to_coordinates_file(
        &self,
        path: impl AsRef<std::path::Path>,
        settings: &tasks::ImageToCoordinates,
    ) -> Result<Solution> {
        self.solve_image_to_coordinates(&Self::file_to_base64(path)?, settings)
            .await
    }

    pub async fn solve_recaptcha_v2(&self, settings: &tasks::RecaptchaV2) -> Result<Solution> {
        self.solve_task(tasks::build_recaptcha_v2(settings, false)?)
            .await
    }

    pub async fn solve_recaptcha_v2_proxy_on(
        &self,
        settings: &tasks::RecaptchaV2,
    ) -> Result<Solution> {
        self.solve_task(tasks::build_recaptcha_v2(settings, true)?)
            .await
    }

    pub async fn solve_recaptcha_v3(&self, settings: &tasks::RecaptchaV3) -> Result<Solution> {
        self.solve_task(tasks::build_recaptcha_v3(settings)?).await
    }

    pub async fn solve_hcaptcha(&self, settings: &tasks::HCaptcha) -> Result<Solution> {
        self.solve_task(tasks::build_hcaptcha(settings, false)?)
            .await
    }

    pub async fn solve_hcaptcha_proxy_on(&self, settings: &tasks::HCaptcha) -> Result<Solution> {
        self.solve_task(tasks::build_hcaptcha(settings, true)?)
            .await
    }

    pub async fn solve_funcaptcha(&self, settings: &tasks::FunCaptcha) -> Result<Solution> {
        self.solve_task(tasks::build_funcaptcha(settings, false)?)
            .await
    }

    pub async fn solve_funcaptcha_proxy_on(
        &self,
        settings: &tasks::FunCaptcha,
    ) -> Result<Solution> {
        self.solve_task(tasks::build_funcaptcha(settings, true)?)
            .await
    }

    pub async fn solve_geetest(&self, settings: &tasks::GeeTest) -> Result<Solution> {
        self.solve_task(tasks::build_geetest(settings, false)?)
            .await
    }

    pub async fn solve_geetest_proxy_on(&self, settings: &tasks::GeeTest) -> Result<Solution> {
        self.solve_task(tasks::build_geetest(settings, true)?).await
    }

    pub async fn solve_turnstile(&self, settings: &tasks::Turnstile) -> Result<Solution> {
        self.solve_task(tasks::build_turnstile(settings, false)?)
            .await
    }

    pub async fn solve_turnstile_proxy_on(&self, settings: &tasks::Turnstile) -> Result<Solution> {
        self.solve_task(tasks::build_turnstile(settings, true)?)
            .await
    }

    pub async fn solve_prosopo(&self, settings: &tasks::Prosopo) -> Result<Solution> {
        self.solve_task(tasks::build_prosopo(settings, false)?)
            .await
    }

    pub async fn solve_prosopo_proxy_on(&self, settings: &tasks::Prosopo) -> Result<Solution> {
        self.solve_task(tasks::build_prosopo(settings, true)?).await
    }

    pub async fn solve_friendly_captcha(
        &self,
        settings: &tasks::FriendlyCaptcha,
    ) -> Result<Solution> {
        self.solve_task(tasks::build_friendly_captcha(settings, false)?)
            .await
    }

    pub async fn solve_friendly_captcha_proxy_on(
        &self,
        settings: &tasks::FriendlyCaptcha,
    ) -> Result<Solution> {
        self.solve_task(tasks::build_friendly_captcha(settings, true)?)
            .await
    }

    pub async fn solve_altcha(&self, settings: &tasks::Altcha) -> Result<Solution> {
        self.solve_task(tasks::build_altcha(settings, false)?).await
    }

    pub async fn solve_altcha_proxy_on(&self, settings: &tasks::Altcha) -> Result<Solution> {
        self.solve_task(tasks::build_altcha(settings, true)?).await
    }

    pub async fn solve_amazon(&self, settings: &tasks::Amazon) -> Result<Solution> {
        self.solve_task(tasks::build_amazon(settings, false)?).await
    }

    pub async fn solve_amazon_proxy_on(&self, settings: &tasks::Amazon) -> Result<Solution> {
        self.solve_task(tasks::build_amazon(settings, true)?).await
    }

    pub async fn solve_antigate(&self, settings: &tasks::AntiGate) -> Result<Solution> {
        self.solve_task(tasks::build_antigate(settings)?).await
    }

    pub async fn solve_antibot_cookie(&self, settings: &tasks::AntiBotCookie) -> Result<Solution> {
        self.solve_task(tasks::build_antibot_cookie(settings)?)
            .await
    }

    // --------------------------------------------------------------- reports

    /// Reports an image captcha as answered incorrectly.
    pub async fn report_incorrect_image_captcha(&self, task_id: i64) -> Result<()> {
        self.report("reportIncorrectImageCaptcha", task_id).await
    }

    pub async fn report_incorrect_recaptcha(&self, task_id: i64) -> Result<()> {
        self.report("reportIncorrectRecaptcha", task_id).await
    }

    pub async fn report_correct_recaptcha(&self, task_id: i64) -> Result<()> {
        self.report("reportCorrectRecaptcha", task_id).await
    }

    pub async fn report_incorrect_hcaptcha(&self, task_id: i64) -> Result<()> {
        self.report("reportIncorrectHcaptcha", task_id).await
    }

    async fn report(&self, method: &str, task_id: i64) -> Result<()> {
        self.json_request(method, json!({ "taskId": task_id }))
            .await?;

        Ok(())
    }

    // --------------------------------------------------------------- helpers

    /// Reads a file and encodes it in base64.
    pub fn file_to_base64(path: impl AsRef<std::path::Path>) -> Result<String> {
        let path = path.as_ref();

        let bytes = std::fs::read(path).map_err(|source| Error::Io {
            path: path.display().to_string(),
            source,
        })?;

        Ok(BASE64.encode(bytes))
    }

    /// Encodes bytes in base64.
    pub fn to_base64(bytes: impl AsRef<[u8]>) -> String {
        BASE64.encode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_the_reference_vectors() {
        // RFC 4648 test vectors
        assert_eq!(Client::to_base64(""), "");
        assert_eq!(Client::to_base64("f"), "Zg==");
        assert_eq!(Client::to_base64("fo"), "Zm8=");
        assert_eq!(Client::to_base64("foo"), "Zm9v");
        assert_eq!(Client::to_base64("foob"), "Zm9vYg==");
        assert_eq!(Client::to_base64("fooba"), "Zm9vYmE=");
        assert_eq!(Client::to_base64("foobar"), "Zm9vYmFy");
        // bytes above 0x7F must not be mangled
        assert_eq!(Client::to_base64([0xFFu8, 0xFE, 0xFD]), "//79");
    }

    #[test]
    fn missing_file_is_an_io_error() {
        let error = Client::file_to_base64("/definitely/not/here.jpg").unwrap_err();

        assert!(matches!(error, Error::Io { .. }));
    }

    #[tokio::test]
    async fn empty_api_key_is_refused_before_any_request() {
        let client = Client::new("").quiet();
        let error = client.get_balance().await.unwrap_err();

        assert!(matches!(error, Error::InvalidTask(_)));
    }

    #[test]
    fn builder_settings_stick() {
        let client = Client::new("KEY")
            .with_soft_id(1187)
            .quiet()
            .with_max_waiting_time(Duration::from_secs(60));

        assert_eq!(client.soft_id, 1187);
        assert!(!client.verbose);
        assert_eq!(client.max_waiting_time, Duration::from_secs(60));
    }
}
