//! The solved captcha.

use serde_json::Value;

/// The `solution` object of a completed task.
///
/// Which accessors return something depends on the task type; see
/// <https://anti-captcha.com/apidoc> for the per-type description. Accessors for
/// fields a task type does not fill return an empty string rather than panicking.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Solution {
    raw: Value,
    task_id: i64,
    cost: f64,
}

impl Solution {
    pub(crate) fn new(raw: Value) -> Self {
        Self {
            raw,
            task_id: 0,
            cost: 0.0,
        }
    }

    pub(crate) fn with_task(mut self, task_id: i64, cost: f64) -> Self {
        self.task_id = task_id;
        self.cost = cost;
        self
    }

    /// Id of the task that produced this solution.
    pub fn task_id(&self) -> i64 {
        self.task_id
    }

    /// What the task cost, in US dollars.
    pub fn cost(&self) -> f64 {
        self.cost
    }

    /// The whole `solution` object, for fields this type does not expose yet.
    pub fn raw(&self) -> &Value {
        &self.raw
    }

    /// Any string field by its API name.
    pub fn get(&self, field: &str) -> Option<&str> {
        self.raw.get(field)?.as_str()
    }

    /// Any field as raw JSON.
    pub fn node(&self, field: &str) -> Option<&Value> {
        match self.raw.get(field) {
            Some(Value::Null) | None => None,
            other => other,
        }
    }

    fn string(&self, field: &str) -> &str {
        self.get(field).unwrap_or_default()
    }

    /// Image captchas.
    pub fn text(&self) -> &str {
        self.string("text")
    }

    /// FunCaptcha, Turnstile, Prosopo, Friendly Captcha, Altcha, Amazon WAF.
    pub fn token(&self) -> &str {
        self.string("token")
    }

    /// Recaptcha and hCaptcha.
    pub fn g_recaptcha_response(&self) -> &str {
        self.string("gRecaptchaResponse")
    }

    /// Recaptcha with `isExtended`.
    pub fn g_recaptcha_response_md5(&self) -> &str {
        self.string("gRecaptchaResponseMd5")
    }

    /// hCaptcha.
    pub fn resp_key(&self) -> &str {
        self.string("respKey")
    }

    /// User-agent of the worker who solved the captcha. Submit the form with it.
    pub fn user_agent(&self) -> &str {
        self.string("userAgent")
    }

    /// AntiGate and AntiBotCookie tasks.
    pub fn url(&self) -> &str {
        self.string("url")
    }

    /// Amazon WAF: the domain the `aws-waf-token` cookie belongs to.
    pub fn domain(&self) -> &str {
        self.string("domain")
    }

    /// GeeTest v3.
    pub fn challenge(&self) -> &str {
        self.string("challenge")
    }

    /// GeeTest v3.
    pub fn seccode(&self) -> &str {
        self.string("seccode")
    }

    /// GeeTest v3.
    pub fn validate(&self) -> &str {
        self.string("validate")
    }

    /// GeeTest v4.
    pub fn captcha_id(&self) -> &str {
        self.string("captcha_id")
    }

    /// GeeTest v4.
    pub fn lot_number(&self) -> &str {
        self.string("lot_number")
    }

    /// GeeTest v4.
    pub fn pass_token(&self) -> &str {
        self.string("pass_token")
    }

    /// GeeTest v4.
    pub fn gen_time(&self) -> &str {
        self.string("gen_time")
    }

    /// GeeTest v4.
    pub fn captcha_output(&self) -> &str {
        self.string("captcha_output")
    }

    /// AntiGate and AntiBotCookie tasks.
    pub fn cookies(&self) -> Option<&Value> {
        self.node("cookies")
    }

    /// AntiGate and AntiBotCookie tasks.
    pub fn local_storage(&self) -> Option<&Value> {
        self.node("localStorage")
    }

    /// AntiGate and AntiBotCookie tasks.
    pub fn fingerprint(&self) -> Option<&Value> {
        self.node("fingerprint")
    }

    /// AntiBotCookie tasks: the headers the worker's browser sent last.
    pub fn last_request_headers(&self) -> Option<&Value> {
        self.node("lastRequestHeaders")
    }

    /// ImageToCoordinates tasks: `[x, y]` points or `[x1, y1, x2, y2]` boxes.
    pub fn coordinates(&self) -> Option<&Value> {
        self.node("coordinates")
    }

    /// The cookies formatted as a `name1=value1; name2=value2` header value.
    pub fn cookie_header(&self) -> String {
        let Some(Value::Object(jar)) = self.cookies() else {
            return String::new();
        };

        jar.iter()
            .map(|(name, value)| match value {
                Value::String(text) => format!("{name}={text}"),
                other => format!("{name}={other}"),
            })
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// The worker's browser user-agent, taken from the AntiBotCookie fingerprint.
    pub fn fingerprint_user_agent(&self) -> &str {
        self.fingerprint()
            .and_then(|value| value.get("self.navigator.userAgent"))
            .and_then(Value::as_str)
            .unwrap_or_default()
    }

    /// True when the API returned a `solution` object without any usable field.
    pub fn is_empty(&self) -> bool {
        match &self.raw {
            Value::Object(fields) => fields.is_empty(),
            Value::Null => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample() -> Solution {
        Solution::new(json!({
            "gRecaptchaResponse": "TOKEN",
            "userAgent": "UA",
            "cookies": {"a": "1", "b": "2"},
            "fingerprint": {"self.navigator.userAgent": "Mozilla/5.0"},
            "coordinates": [[10, 20], [30, 40]],
            "gen_time": "1692000000",
            "nothing": null
        }))
    }

    #[test]
    fn reads_known_fields() {
        let solution = sample();

        assert_eq!(solution.g_recaptcha_response(), "TOKEN");
        assert_eq!(solution.user_agent(), "UA");
        assert_eq!(solution.gen_time(), "1692000000");
        assert_eq!(solution.fingerprint_user_agent(), "Mozilla/5.0");
    }

    #[test]
    fn missing_fields_are_empty_not_a_panic() {
        let solution = sample();

        assert_eq!(solution.text(), "");
        assert_eq!(solution.token(), "");
        assert!(solution.node("nope").is_none());
        // an explicit JSON null counts as absent
        assert!(solution.node("nothing").is_none());
    }

    #[test]
    fn builds_a_cookie_header() {
        assert_eq!(sample().cookie_header(), "a=1; b=2");
        assert_eq!(Solution::default().cookie_header(), "");
    }

    #[test]
    fn exposes_json_nodes() {
        let solution = sample();

        assert_eq!(solution.coordinates().unwrap().as_array().unwrap().len(), 2);
        assert!(solution.cookies().unwrap().is_object());
    }

    #[test]
    fn default_is_empty() {
        let solution = Solution::default();

        assert!(solution.is_empty());
        assert_eq!(solution.token(), "");
    }
}
