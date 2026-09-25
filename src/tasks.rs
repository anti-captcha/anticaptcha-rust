//! Task settings and the JSON they turn into.
//!
//! Every struct implements [`Default`], so you only fill what you need:
//!
//! ```
//! use anticaptcha::RecaptchaV2;
//!
//! let params = RecaptchaV2 {
//!     website_url: "https://website.com/".into(),
//!     website_key: "SITE_KEY".into(),
//!     is_invisible: true,
//!     ..Default::default()
//! };
//! ```

use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

use crate::error::{Error, Result};

/// Which protocol your proxy speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProxyType {
    #[default]
    Http,
    Socks4,
    Socks5,
}

impl ProxyType {
    fn as_str(self) -> &'static str {
        match self {
            ProxyType::Http => "http",
            ProxyType::Socks4 => "socks4",
            ProxyType::Socks5 => "socks5",
        }
    }
}

/// Your proxy, for the `*_proxy_on` methods.
///
/// Do not use purchased or rented proxies from proxy services, use proper proxy
/// software like Squid:
/// <https://anti-captcha.com/apidoc/articles/how-to-install-squid>
#[derive(Debug, Clone, Default)]
pub struct Proxy {
    pub proxy_type: ProxyType,
    pub address: String,
    pub port: u16,
    /// Optional.
    pub login: String,
    /// Optional.
    pub password: String,
}

impl Proxy {
    /// A proxy without authentication.
    pub fn new(proxy_type: ProxyType, address: impl Into<String>, port: u16) -> Self {
        Self {
            proxy_type,
            address: address.into(),
            port,
            ..Default::default()
        }
    }

    /// Adds the login and password.
    pub fn with_auth(mut self, login: impl Into<String>, password: impl Into<String>) -> Self {
        self.login = login.into();
        self.password = password.into();
        self
    }
}

/// <https://anti-captcha.com/apidoc/task-types/ImageToTextTask>
#[derive(Debug, Clone, Default)]
pub struct ImageSettings {
    /// The image contains 2 or more words.
    pub phrase: bool,
    /// The answer is case sensitive.
    pub case_sensitive: bool,
    /// `0` no requirements, `1` digits only, `2` no digits.
    pub numeric: u8,
    /// The answer is the result of a math operation, like `50+5`.
    pub math_operation: bool,
    /// `0` for no limit.
    pub min_length: u32,
    /// `0` for no limit.
    pub max_length: u32,
    /// `"en"` or `"rn"`. Defaults to `"en"` when left empty.
    pub language_pool: String,
    /// Hint for the worker.
    pub comment: String,
    /// Optional, groups the dashboard statistics by website.
    pub website_url: String,
}

/// <https://anti-captcha.com/apidoc/task-types/ImageToCoordinatesTask>
#[derive(Debug, Clone, Default)]
pub struct ImageToCoordinates {
    /// `"points"` or `"rectangles"`. Defaults to `"points"` when left empty.
    pub mode: String,
    /// Instruction for the worker.
    pub comment: String,
    /// Optional, groups the dashboard statistics by website.
    pub website_url: String,
}

/// <https://anti-captcha.com/apidoc/task-types/RecaptchaV2TaskProxyless>
#[derive(Debug, Clone, Default)]
pub struct RecaptchaV2 {
    pub website_url: String,
    /// The `data-sitekey` value.
    pub website_key: String,
    pub website_s_token: String,
    /// For Enterprise tasks it is sent only when `true`.
    pub is_invisible: bool,
    /// The `data-s` parameter, typical for google.com websites.
    pub data_s_value: String,
    /// Proxy-on tasks only.
    pub user_agent: String,
    /// Switches the task to `RecaptchaV2EnterpriseTask`.
    pub is_enterprise: bool,
    /// Parameters passed to the `grecaptcha.enterprise.render` call.
    pub enterprise_payload: BTreeMap<String, String>,
    /// e.g. `"recaptcha.net"`, when the script does not come from google.com.
    pub api_domain: String,
    pub proxy: Proxy,
}

/// <https://anti-captcha.com/apidoc/task-types/RecaptchaV3TaskProxyless>
///
/// This captcha has no proxy-on version.
#[derive(Debug, Clone)]
pub struct RecaptchaV3 {
    pub website_url: String,
    pub website_key: String,
    /// One of `0.3`, `0.7`, `0.9`.
    pub min_score: f64,
    pub page_action: String,
    pub is_enterprise: bool,
    pub api_domain: String,
}

impl Default for RecaptchaV3 {
    fn default() -> Self {
        Self {
            website_url: String::new(),
            website_key: String::new(),
            min_score: 0.3,
            page_action: String::new(),
            is_enterprise: false,
            api_domain: String::new(),
        }
    }
}

/// <https://anti-captcha.com/apidoc/task-types/HCaptchaTaskProxyless>
#[derive(Debug, Clone, Default)]
pub struct HCaptcha {
    pub website_url: String,
    pub website_key: String,
    pub is_invisible: bool,
    pub is_enterprise: bool,
    /// `rqdata`, `sentry`, `apiEndpoint`, `endpoint`, `reportapi`, `assethost`, `imghost`.
    pub enterprise_payload: BTreeMap<String, String>,
    /// Proxy-on tasks only.
    pub user_agent: String,
    /// Proxy-on tasks only, `name1=value1; name2=value2`.
    pub cookies: String,
    pub proxy: Proxy,
}

/// <https://anti-captcha.com/apidoc/task-types/FunCaptchaTaskProxyless>
#[derive(Debug, Clone, Default)]
pub struct FunCaptcha {
    pub website_url: String,
    /// The `data-pkey` value.
    pub website_public_key: String,
    /// e.g. `"somewebsite-api.arkoselabs.com"`.
    pub api_subdomain: String,
    /// The `blob` value, as a JSON string.
    pub data_blob: String,
    pub user_agent: String,
    pub proxy: Proxy,
}

/// <https://anti-captcha.com/apidoc/task-types/GeeTestTaskProxyless>
#[derive(Debug, Clone)]
pub struct GeeTest {
    pub website_url: String,
    /// v3: the `gt` key, v4: the `captcha_id` value.
    pub gt: String,
    /// v3 only, a one-time value.
    pub challenge: String,
    pub api_subdomain: String,
    /// `3` or `4`.
    pub version: u8,
    /// v4 only, e.g. `{"riskType": "slide"}`.
    pub init_parameters: Map<String, Value>,
    pub user_agent: String,
    pub proxy: Proxy,
}

impl Default for GeeTest {
    fn default() -> Self {
        Self {
            website_url: String::new(),
            gt: String::new(),
            challenge: String::new(),
            api_subdomain: String::new(),
            version: 3,
            init_parameters: Map::new(),
            user_agent: String::new(),
            proxy: Proxy::default(),
        }
    }
}

/// <https://anti-captcha.com/apidoc/task-types/TurnstileTaskProxyless>
#[derive(Debug, Clone, Default)]
pub struct Turnstile {
    pub website_url: String,
    pub website_key: String,
    pub action: String,
    pub c_data: String,
    pub chl_page_data: String,
    pub user_agent: String,
    pub proxy: Proxy,
}

/// <https://anti-captcha.com/apidoc/task-types/ProsopoTaskProxyless>
#[derive(Debug, Clone, Default)]
pub struct Prosopo {
    pub website_url: String,
    pub website_key: String,
    pub user_agent: String,
    pub proxy: Proxy,
}

/// <https://anti-captcha.com/apidoc/task-types/FriendlyCaptchaTaskProxyless>
#[derive(Debug, Clone, Default)]
pub struct FriendlyCaptcha {
    pub website_url: String,
    pub website_key: String,
    pub user_agent: String,
    pub proxy: Proxy,
}

/// <https://anti-captcha.com/apidoc/task-types/AltchaTaskProxyless>
#[derive(Debug, Clone, Default)]
pub struct Altcha {
    pub website_url: String,
    /// Use this or [`Altcha::challenge_json`].
    pub challenge_url: String,
    /// Use this or [`Altcha::challenge_url`].
    pub challenge_json: String,
    pub user_agent: String,
    pub proxy: Proxy,
}

/// <https://anti-captcha.com/apidoc/task-types/AmazonTaskProxyless>
#[derive(Debug, Clone, Default)]
pub struct Amazon {
    pub website_url: String,
    /// `key` from `window.gokuProps`, or the widget API key.
    pub website_key: String,
    pub iv: String,
    pub context: String,
    pub captcha_script: String,
    pub challenge_script: String,
    /// Required when [`Amazon::waf_type`] is `"widget"`.
    pub jsapi_script: String,
    /// `"widget"` for a standalone widget, empty for the bot filtering page.
    pub waf_type: String,
    pub user_agent: String,
    pub proxy: Proxy,
}

/// <https://anti-captcha.com/apidoc/task-types/AntiGateTask>
#[derive(Debug, Clone, Default)]
pub struct AntiGate {
    pub website_url: String,
    /// <https://anti-captcha.com/apidoc/antigate-templates>
    pub template_name: String,
    pub variables: Map<String, Value>,
    pub domains_of_interest: Vec<String>,
    /// The proxy is optional for this task type.
    pub use_proxy: bool,
    pub proxy: Proxy,
}

/// <https://anti-captcha.com/apidoc/task-types/AntiBotCookieTask>
#[derive(Debug, Clone, Default)]
pub struct AntiBotCookie {
    pub website_url: String,
    /// Required: the cookies are only valid for this IP address.
    pub proxy: Proxy,
}

// --------------------------------------------------------------- task builders

fn set_if_not_empty(task: &mut Map<String, Value>, name: &str, value: &str) {
    if !value.is_empty() {
        task.insert(name.to_owned(), json!(value));
    }
}

fn set_payload(task: &mut Map<String, Value>, name: &str, payload: &BTreeMap<String, String>) {
    if !payload.is_empty() {
        task.insert(name.to_owned(), json!(payload));
    }
}

fn object(pairs: Vec<(&str, Value)>) -> Map<String, Value> {
    pairs
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect()
}

/// Validates the proxy and appends it to the task.
fn add_proxy(task: &mut Map<String, Value>, proxy: &Proxy) -> Result<()> {
    if proxy.address.is_empty() {
        return Err(Error::InvalidTask(
            "proxy address is empty, it is required for a proxy-on task".into(),
        ));
    }

    if proxy.port == 0 {
        return Err(Error::InvalidTask("proxy port is not set".into()));
    }

    task.insert("proxyType".into(), json!(proxy.proxy_type.as_str()));
    task.insert("proxyAddress".into(), json!(proxy.address));
    task.insert("proxyPort".into(), json!(proxy.port));

    if !proxy.login.is_empty() {
        task.insert("proxyLogin".into(), json!(proxy.login));
        task.insert("proxyPassword".into(), json!(proxy.password));
    }

    Ok(())
}

pub(crate) fn build_image(body: &str, settings: &ImageSettings) -> Result<Value> {
    if body.is_empty() {
        return Err(Error::InvalidTask("captcha image is empty".into()));
    }

    let language_pool = if settings.language_pool.is_empty() {
        "en"
    } else {
        &settings.language_pool
    };

    let mut task = object(vec![
        ("type", json!("ImageToTextTask")),
        ("body", json!(body)),
        ("phrase", json!(settings.phrase)),
        ("case", json!(settings.case_sensitive)),
        ("numeric", json!(settings.numeric)),
        ("math", json!(settings.math_operation)),
        ("minLength", json!(settings.min_length)),
        ("maxLength", json!(settings.max_length)),
        ("languagePool", json!(language_pool)),
    ]);

    set_if_not_empty(&mut task, "comment", &settings.comment);
    set_if_not_empty(&mut task, "websiteURL", &settings.website_url);

    Ok(Value::Object(task))
}

pub(crate) fn build_image_to_coordinates(
    body: &str,
    settings: &ImageToCoordinates,
) -> Result<Value> {
    if body.is_empty() {
        return Err(Error::InvalidTask("captcha image is empty".into()));
    }

    let mode = if settings.mode.is_empty() {
        "points"
    } else {
        &settings.mode
    };

    let mut task = object(vec![
        ("type", json!("ImageToCoordinatesTask")),
        ("body", json!(body)),
        ("mode", json!(mode)),
    ]);

    set_if_not_empty(&mut task, "comment", &settings.comment);
    set_if_not_empty(&mut task, "websiteURL", &settings.website_url);

    Ok(Value::Object(task))
}

pub(crate) fn build_recaptcha_v2(settings: &RecaptchaV2, proxy_on: bool) -> Result<Value> {
    let task_type = match (settings.is_enterprise, proxy_on) {
        (true, true) => "RecaptchaV2EnterpriseTask",
        (true, false) => "RecaptchaV2EnterpriseTaskProxyless",
        (false, true) => "RecaptchaV2Task",
        (false, false) => "RecaptchaV2TaskProxyless",
    };

    let mut task = object(vec![
        ("type", json!(task_type)),
        ("websiteURL", json!(settings.website_url)),
        ("websiteKey", json!(settings.website_key)),
    ]);

    // Enterprise tasks take isInvisible only when it is true.
    if !settings.is_enterprise || settings.is_invisible {
        task.insert("isInvisible".into(), json!(settings.is_invisible));
    }

    set_if_not_empty(&mut task, "websiteSToken", &settings.website_s_token);
    set_if_not_empty(&mut task, "recaptchaDataSValue", &settings.data_s_value);
    set_if_not_empty(&mut task, "apiDomain", &settings.api_domain);
    set_payload(&mut task, "enterprisePayload", &settings.enterprise_payload);

    if proxy_on {
        add_proxy(&mut task, &settings.proxy)?;
        set_if_not_empty(&mut task, "userAgent", &settings.user_agent);
    }

    Ok(Value::Object(task))
}

pub(crate) fn build_recaptcha_v3(settings: &RecaptchaV3) -> Result<Value> {
    if ![0.3, 0.7, 0.9].contains(&settings.min_score) {
        return Err(Error::InvalidTask(format!(
            "min_score must be one of 0.3, 0.7, 0.9; got {}",
            settings.min_score
        )));
    }

    let mut task = object(vec![
        ("type", json!("RecaptchaV3TaskProxyless")),
        ("websiteURL", json!(settings.website_url)),
        ("websiteKey", json!(settings.website_key)),
        ("minScore", json!(settings.min_score)),
        ("isEnterprise", json!(settings.is_enterprise)),
    ]);

    set_if_not_empty(&mut task, "pageAction", &settings.page_action);
    set_if_not_empty(&mut task, "apiDomain", &settings.api_domain);

    Ok(Value::Object(task))
}

pub(crate) fn build_hcaptcha(settings: &HCaptcha, proxy_on: bool) -> Result<Value> {
    let mut task = object(vec![
        (
            "type",
            json!(if proxy_on {
                "HCaptchaTask"
            } else {
                "HCaptchaTaskProxyless"
            }),
        ),
        ("websiteURL", json!(settings.website_url)),
        ("websiteKey", json!(settings.website_key)),
        ("isInvisible", json!(settings.is_invisible)),
        ("isEnterprise", json!(settings.is_enterprise)),
    ]);

    set_payload(&mut task, "enterprisePayload", &settings.enterprise_payload);

    if proxy_on {
        add_proxy(&mut task, &settings.proxy)?;
        set_if_not_empty(&mut task, "userAgent", &settings.user_agent);
        set_if_not_empty(&mut task, "cookies", &settings.cookies);
    }

    Ok(Value::Object(task))
}

pub(crate) fn build_funcaptcha(settings: &FunCaptcha, proxy_on: bool) -> Result<Value> {
    let mut task = object(vec![
        (
            "type",
            json!(if proxy_on {
                "FunCaptchaTask"
            } else {
                "FunCaptchaTaskProxyless"
            }),
        ),
        ("websiteURL", json!(settings.website_url)),
        ("websitePublicKey", json!(settings.website_public_key)),
    ]);

    set_if_not_empty(
        &mut task,
        "funcaptchaApiJSSubdomain",
        &settings.api_subdomain,
    );
    set_if_not_empty(&mut task, "data", &settings.data_blob);

    if proxy_on {
        add_proxy(&mut task, &settings.proxy)?;
        set_if_not_empty(&mut task, "userAgent", &settings.user_agent);
    }

    Ok(Value::Object(task))
}

pub(crate) fn build_geetest(settings: &GeeTest, proxy_on: bool) -> Result<Value> {
    if settings.version != 3 && settings.version != 4 {
        return Err(Error::InvalidTask(format!(
            "GeeTest version must be 3 or 4, got {}",
            settings.version
        )));
    }

    if settings.version == 3 && settings.challenge.is_empty() {
        return Err(Error::InvalidTask(
            "GeeTest v3 requires a challenge value".into(),
        ));
    }

    let mut task = object(vec![
        (
            "type",
            json!(if proxy_on {
                "GeeTestTask"
            } else {
                "GeeTestTaskProxyless"
            }),
        ),
        ("websiteURL", json!(settings.website_url)),
        ("gt", json!(settings.gt)),
        ("version", json!(settings.version)),
    ]);

    set_if_not_empty(&mut task, "challenge", &settings.challenge);
    set_if_not_empty(
        &mut task,
        "geetestApiServerSubdomain",
        &settings.api_subdomain,
    );

    if !settings.init_parameters.is_empty() {
        task.insert(
            "initParameters".into(),
            Value::Object(settings.init_parameters.clone()),
        );
    }

    if proxy_on {
        add_proxy(&mut task, &settings.proxy)?;
        set_if_not_empty(&mut task, "userAgent", &settings.user_agent);
    }

    Ok(Value::Object(task))
}

pub(crate) fn build_turnstile(settings: &Turnstile, proxy_on: bool) -> Result<Value> {
    let mut task = object(vec![
        (
            "type",
            json!(if proxy_on {
                "TurnstileTask"
            } else {
                "TurnstileTaskProxyless"
            }),
        ),
        ("websiteURL", json!(settings.website_url)),
        ("websiteKey", json!(settings.website_key)),
    ]);

    set_if_not_empty(&mut task, "action", &settings.action);
    set_if_not_empty(&mut task, "cData", &settings.c_data);
    set_if_not_empty(&mut task, "chlPageData", &settings.chl_page_data);

    if proxy_on {
        add_proxy(&mut task, &settings.proxy)?;
        set_if_not_empty(&mut task, "userAgent", &settings.user_agent);
    }

    Ok(Value::Object(task))
}

/// Prosopo and Friendly Captcha take exactly the same parameters.
fn build_sitekey_task(
    task_type: &str,
    website_url: &str,
    website_key: &str,
    user_agent: &str,
    proxy: Option<&Proxy>,
) -> Result<Value> {
    let mut task = object(vec![
        ("type", json!(task_type)),
        ("websiteURL", json!(website_url)),
        ("websiteKey", json!(website_key)),
    ]);

    if let Some(proxy) = proxy {
        add_proxy(&mut task, proxy)?;
        set_if_not_empty(&mut task, "userAgent", user_agent);
    }

    Ok(Value::Object(task))
}

pub(crate) fn build_prosopo(settings: &Prosopo, proxy_on: bool) -> Result<Value> {
    build_sitekey_task(
        if proxy_on {
            "ProsopoTask"
        } else {
            "ProsopoTaskProxyless"
        },
        &settings.website_url,
        &settings.website_key,
        &settings.user_agent,
        proxy_on.then_some(&settings.proxy),
    )
}

pub(crate) fn build_friendly_captcha(settings: &FriendlyCaptcha, proxy_on: bool) -> Result<Value> {
    build_sitekey_task(
        if proxy_on {
            "FriendlyCaptchaTask"
        } else {
            "FriendlyCaptchaTaskProxyless"
        },
        &settings.website_url,
        &settings.website_key,
        &settings.user_agent,
        proxy_on.then_some(&settings.proxy),
    )
}

pub(crate) fn build_altcha(settings: &Altcha, proxy_on: bool) -> Result<Value> {
    if settings.challenge_url.is_empty() && settings.challenge_json.is_empty() {
        return Err(Error::InvalidTask(
            "set either challenge_url or challenge_json".into(),
        ));
    }

    let mut task = object(vec![
        (
            "type",
            json!(if proxy_on {
                "AltchaTask"
            } else {
                "AltchaTaskProxyless"
            }),
        ),
        ("websiteURL", json!(settings.website_url)),
    ]);

    set_if_not_empty(&mut task, "challengeURL", &settings.challenge_url);
    set_if_not_empty(&mut task, "challengeJSON", &settings.challenge_json);

    if proxy_on {
        add_proxy(&mut task, &settings.proxy)?;
        set_if_not_empty(&mut task, "userAgent", &settings.user_agent);
    }

    Ok(Value::Object(task))
}

pub(crate) fn build_amazon(settings: &Amazon, proxy_on: bool) -> Result<Value> {
    let mut task = object(vec![
        (
            "type",
            json!(if proxy_on {
                "AmazonTask"
            } else {
                "AmazonTaskProxyless"
            }),
        ),
        ("websiteURL", json!(settings.website_url)),
        ("websiteKey", json!(settings.website_key)),
    ]);

    set_if_not_empty(&mut task, "wafType", &settings.waf_type);
    set_if_not_empty(&mut task, "iv", &settings.iv);
    set_if_not_empty(&mut task, "context", &settings.context);
    set_if_not_empty(&mut task, "captchaScript", &settings.captcha_script);
    set_if_not_empty(&mut task, "challengeScript", &settings.challenge_script);
    set_if_not_empty(&mut task, "jsapiScript", &settings.jsapi_script);

    if proxy_on {
        add_proxy(&mut task, &settings.proxy)?;
        set_if_not_empty(&mut task, "userAgent", &settings.user_agent);
    }

    Ok(Value::Object(task))
}

pub(crate) fn build_antigate(settings: &AntiGate) -> Result<Value> {
    let mut task = object(vec![
        ("type", json!("AntiGateTask")),
        ("websiteURL", json!(settings.website_url)),
        ("templateName", json!(settings.template_name)),
    ]);

    if !settings.variables.is_empty() {
        task.insert(
            "variables".into(),
            Value::Object(settings.variables.clone()),
        );
    }

    if !settings.domains_of_interest.is_empty() {
        task.insert(
            "domainsOfInterest".into(),
            json!(settings.domains_of_interest),
        );
    }

    if settings.use_proxy {
        add_proxy(&mut task, &settings.proxy)?;
    }

    Ok(Value::Object(task))
}

pub(crate) fn build_antibot_cookie(settings: &AntiBotCookie) -> Result<Value> {
    let mut task = object(vec![
        ("type", json!("AntiBotCookieTask")),
        ("websiteURL", json!(settings.website_url)),
    ]);

    add_proxy(&mut task, &settings.proxy)?;

    // This task type takes no proxyType, only http proxies are supported.
    task.remove("proxyType");

    Ok(Value::Object(task))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proxy() -> Proxy {
        Proxy::new(ProxyType::Socks5, "1.2.3.4", 1234).with_auth("login", "password")
    }

    #[test]
    fn image_task() {
        let settings = ImageSettings {
            comment: "type in green".into(),
            numeric: 1,
            ..Default::default()
        };

        assert_eq!(
            build_image("QUFB", &settings).unwrap(),
            json!({
                "type": "ImageToTextTask",
                "body": "QUFB",
                "phrase": false,
                "case": false,
                "numeric": 1,
                "math": false,
                "minLength": 0,
                "maxLength": 0,
                "languagePool": "en",
                "comment": "type in green"
            })
        );

        assert!(build_image("", &ImageSettings::default()).is_err());
    }

    #[test]
    fn image_to_coordinates_defaults_to_points() {
        let task = build_image_to_coordinates("QUFB", &ImageToCoordinates::default()).unwrap();

        assert_eq!(task["mode"], "points");
        assert_eq!(task["type"], "ImageToCoordinatesTask");
    }

    #[test]
    fn recaptcha_v2_proxyless() {
        let settings = RecaptchaV2 {
            website_url: "https://website.com/".into(),
            website_key: "KEY".into(),
            is_invisible: true,
            ..Default::default()
        };

        assert_eq!(
            build_recaptcha_v2(&settings, false).unwrap(),
            json!({
                "type": "RecaptchaV2TaskProxyless",
                "websiteURL": "https://website.com/",
                "websiteKey": "KEY",
                "isInvisible": true
            })
        );
    }

    #[test]
    fn recaptcha_v2_proxy_on() {
        let settings = RecaptchaV2 {
            website_url: "https://website.com/".into(),
            website_key: "KEY".into(),
            user_agent: "UA".into(),
            proxy: proxy(),
            ..Default::default()
        };

        assert_eq!(
            build_recaptcha_v2(&settings, true).unwrap(),
            json!({
                "type": "RecaptchaV2Task",
                "websiteURL": "https://website.com/",
                "websiteKey": "KEY",
                "isInvisible": false,
                "proxyType": "socks5",
                "proxyAddress": "1.2.3.4",
                "proxyPort": 1234,
                "proxyLogin": "login",
                "proxyPassword": "password",
                "userAgent": "UA"
            })
        );
    }

    #[test]
    fn recaptcha_v2_enterprise_switches_the_type() {
        let mut payload = BTreeMap::new();
        payload.insert("s".to_owned(), "TOKEN".to_owned());

        let settings = RecaptchaV2 {
            website_url: "https://website.com/".into(),
            website_key: "KEY".into(),
            is_enterprise: true,
            enterprise_payload: payload,
            api_domain: "recaptcha.net".into(),
            ..Default::default()
        };

        let task = build_recaptcha_v2(&settings, false).unwrap();

        assert_eq!(task["type"], "RecaptchaV2EnterpriseTaskProxyless");
        assert_eq!(task["enterprisePayload"]["s"], "TOKEN");
        assert_eq!(task["apiDomain"], "recaptcha.net");
        assert!(task.get("isInvisible").is_none());

        let invisible = RecaptchaV2 {
            is_invisible: true,
            proxy: proxy(),
            ..settings
        };
        let task = build_recaptcha_v2(&invisible, true).unwrap();

        assert_eq!(task["type"], "RecaptchaV2EnterpriseTask");
        assert_eq!(task["isInvisible"], true);
    }

    #[test]
    fn recaptcha_v3_checks_the_score() {
        let settings = RecaptchaV3 {
            website_url: "https://website.com/".into(),
            website_key: "KEY".into(),
            page_action: "login".into(),
            min_score: 0.7,
            ..Default::default()
        };

        let task = build_recaptcha_v3(&settings).unwrap();
        assert_eq!(task["minScore"], 0.7);
        assert_eq!(task["pageAction"], "login");

        let broken = RecaptchaV3 {
            min_score: 0.55,
            ..settings
        };
        assert!(build_recaptcha_v3(&broken).is_err());
    }

    #[test]
    fn hcaptcha_sends_lowercase_is_enterprise() {
        let settings = HCaptcha {
            website_url: "https://website.com/".into(),
            website_key: "KEY".into(),
            is_enterprise: true,
            ..Default::default()
        };

        let task = build_hcaptcha(&settings, false).unwrap();

        assert_eq!(task["isEnterprise"], true);
        assert!(task.get("IsEnterprise").is_none());
    }

    #[test]
    fn funcaptcha_keeps_the_blob_a_string() {
        let settings = FunCaptcha {
            website_url: "https://website.com/".into(),
            website_public_key: "KEY".into(),
            api_subdomain: "x.arkoselabs.com".into(),
            data_blob: r#"{"blob":"B"}"#.into(),
            proxy: Proxy::new(ProxyType::Http, "1.2.3.4", 8080),
            ..Default::default()
        };

        let task = build_funcaptcha(&settings, true).unwrap();

        assert_eq!(task["type"], "FunCaptchaTask");
        assert_eq!(task["data"], r#"{"blob":"B"}"#);
        assert_eq!(task["funcaptchaApiJSSubdomain"], "x.arkoselabs.com");
        // an empty login must not be sent
        assert!(task.get("proxyLogin").is_none());
    }

    #[test]
    fn geetest_versions() {
        let v3 = GeeTest {
            website_url: "https://website.com/".into(),
            gt: "GT".into(),
            challenge: "CHALLENGE".into(),
            ..Default::default()
        };
        let task = build_geetest(&v3, false).unwrap();
        assert_eq!(task["version"], 3);
        assert_eq!(task["challenge"], "CHALLENGE");

        let mut init = Map::new();
        init.insert("riskType".into(), json!("slide"));

        let v4 = GeeTest {
            website_url: "https://website.com/".into(),
            gt: "GT".into(),
            version: 4,
            init_parameters: init,
            proxy: Proxy::new(ProxyType::Http, "1.2.3.4", 8080),
            ..Default::default()
        };
        let task = build_geetest(&v4, true).unwrap();
        assert_eq!(task["type"], "GeeTestTask");
        assert_eq!(task["version"], 4);
        assert_eq!(task["initParameters"]["riskType"], "slide");
        assert!(task.get("challenge").is_none());

        let no_challenge = GeeTest {
            gt: "GT".into(),
            ..Default::default()
        };
        assert!(build_geetest(&no_challenge, false).is_err());

        let bad_version = GeeTest {
            gt: "GT".into(),
            version: 5,
            ..Default::default()
        };
        assert!(build_geetest(&bad_version, false).is_err());
    }

    #[test]
    fn turnstile_and_sitekey_tasks() {
        let turnstile = Turnstile {
            website_url: "https://website.com/".into(),
            website_key: "0x4".into(),
            action: "login".into(),
            ..Default::default()
        };

        assert_eq!(
            build_turnstile(&turnstile, false).unwrap(),
            json!({
                "type": "TurnstileTaskProxyless",
                "websiteURL": "https://website.com/",
                "websiteKey": "0x4",
                "action": "login"
            })
        );

        let prosopo = Prosopo {
            website_url: "https://website.com/".into(),
            website_key: "KEY".into(),
            proxy: proxy(),
            ..Default::default()
        };
        assert_eq!(
            build_prosopo(&prosopo, false).unwrap()["type"],
            "ProsopoTaskProxyless"
        );
        assert_eq!(
            build_prosopo(&prosopo, true).unwrap()["type"],
            "ProsopoTask"
        );

        let friendly = FriendlyCaptcha {
            website_url: "https://website.com/".into(),
            website_key: "KEY".into(),
            ..Default::default()
        };
        assert_eq!(
            build_friendly_captcha(&friendly, false).unwrap()["type"],
            "FriendlyCaptchaTaskProxyless"
        );
    }

    #[test]
    fn altcha_needs_a_challenge() {
        let settings = Altcha {
            website_url: "https://website.com/".into(),
            challenge_url: "/challenge".into(),
            ..Default::default()
        };

        assert_eq!(
            build_altcha(&settings, false).unwrap(),
            json!({
                "type": "AltchaTaskProxyless",
                "websiteURL": "https://website.com/",
                "challengeURL": "/challenge"
            })
        );

        let empty = Altcha {
            website_url: "https://website.com/".into(),
            ..Default::default()
        };
        assert!(build_altcha(&empty, false).is_err());
    }

    #[test]
    fn amazon_widget() {
        let settings = Amazon {
            website_url: "https://website.com/".into(),
            website_key: "KEY".into(),
            waf_type: "widget".into(),
            jsapi_script: "https://x/jsapi.js".into(),
            ..Default::default()
        };

        assert_eq!(
            build_amazon(&settings, false).unwrap(),
            json!({
                "type": "AmazonTaskProxyless",
                "websiteURL": "https://website.com/",
                "websiteKey": "KEY",
                "wafType": "widget",
                "jsapiScript": "https://x/jsapi.js"
            })
        );
    }

    #[test]
    fn antigate_proxy_is_optional() {
        let mut variables = Map::new();
        variables.insert("login".into(), json!("value"));

        let settings = AntiGate {
            website_url: "http://website.com/".into(),
            template_name: "Template".into(),
            variables,
            domains_of_interest: vec!["example.com".into()],
            use_proxy: true,
            proxy: Proxy::new(ProxyType::Http, "1.2.3.4", 1234),
        };

        let task = build_antigate(&settings).unwrap();
        assert_eq!(task["variables"]["login"], "value");
        assert_eq!(task["domainsOfInterest"], json!(["example.com"]));
        assert_eq!(task["proxyType"], "http");

        let without = AntiGate {
            use_proxy: false,
            ..settings
        };
        assert!(build_antigate(&without)
            .unwrap()
            .get("proxyAddress")
            .is_none());
    }

    #[test]
    fn antibot_cookie_sends_no_proxy_type() {
        let settings = AntiBotCookie {
            website_url: "https://website.com/".into(),
            proxy: Proxy::new(ProxyType::Http, "1.2.3.4", 3128).with_auth("login", "password"),
        };

        assert_eq!(
            build_antibot_cookie(&settings).unwrap(),
            json!({
                "type": "AntiBotCookieTask",
                "websiteURL": "https://website.com/",
                "proxyAddress": "1.2.3.4",
                "proxyPort": 3128,
                "proxyLogin": "login",
                "proxyPassword": "password"
            })
        );
    }

    #[test]
    fn proxy_is_validated() {
        let no_address = RecaptchaV2 {
            website_url: "https://website.com/".into(),
            website_key: "KEY".into(),
            ..Default::default()
        };
        assert!(build_recaptcha_v2(&no_address, true).is_err());

        let no_port = RecaptchaV2 {
            proxy: Proxy::new(ProxyType::Http, "1.2.3.4", 0),
            ..no_address.clone()
        };
        assert!(build_recaptcha_v2(&no_port, true).is_err());

        let good = RecaptchaV2 {
            proxy: Proxy::new(ProxyType::Http, "1.2.3.4", 8080),
            ..no_address
        };
        assert!(build_recaptcha_v2(&good, true).is_ok());
    }
}
