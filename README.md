## Official Anti-Captcha.com Rust crate ##

Official anti-captcha.com Rust crate for solving images with text, Recaptcha v2/v3 Enterprise/non-Enterprise, Funcaptcha, GeeTest, HCaptcha Enterprise/non-Enterprise, Turnstile, Amazon WAF, Prosopo, Friendly Captcha and Altcha.

[Anti-captcha](https://anti-captcha.com) is an oldest and cheapest web service dedicated to solving captchas by human workers from around the world. By solving captchas with us you help people in poorest regions of the world to earn money, which not only cover their basic needs, but also gives them ability to financially help their families, study and avoid jobs where they're simply not happy.

To use the service you need to [register](https://anti-captcha.com/clients/) and topup your balance. Prices start from $0.0005 per image captcha and $0.002 for Recaptcha. That's $0.5 per 1000 for images and $2 for 1000 Recaptchas.

For more technical information and articles visit our [documentation](https://anti-captcha.com/apidoc) page.

**Install the crate**:
```bash
cargo add anticaptchaofficial
```
or add it to `Cargo.toml` by hand:
```toml
[dependencies]
anticaptchaofficial = "1"
```

The crate is called `anticaptchaofficial`, the library you `use` is `anticaptcha`:
```rust
use anticaptcha::{Client, RecaptchaV2};
```

The API is async and needs a runtime, [tokio](https://tokio.rs) by default. If your program is not async, turn on the [`blocking`](#synchronous-api) feature instead.

Requires Rust 1.86 or newer. The crate's own code needs much less, the floor comes from `reqwest` pulling in `idna`/`icu`; on an older toolchain Cargo's MSRV-aware resolver may still find a working combination.

Every settings struct implements `Default`, so you only fill what you need and finish with `..Default::default()`.

**Examples how to solve:**

- [Image Captcha](#solve-image-captcha)
- [Recaptcha V2](#solve-recaptcha-v2)
- [Recaptcha V2 Enterprise](#solve-recaptcha-v2-enterprise)
- [Recaptcha V3](#solve-recaptcha-v3)
- [hCaptcha](#solve-hcaptcha)
- [FunCaptcha](#solve-funcaptcha)
- [GeeTest](#solve-geetest)
- [Turnstile](#solve-turnstile)
- [Image to coordinates](#image-to-coordinates)
- [AntiGate (custom tasks)](#solve-antigate-custom-tasks)
- [AntiBot cookies](#get-antibot-cookies)
- [Prosopo](#solve-prosopo)
- [Friendly Captcha](#solve-friendly-captcha)
- [Amazon WAF](#solve-amazon-waf)
- [Altcha](#solve-altcha)

### Solve image captcha
```rust
use anticaptcha::{Client, ImageSettings};

#[tokio::main]
async fn main() -> anticaptcha::Result<()> {
    // Create the API client and set the API key
    let ac = Client::new("API_KEY_HERE")
        // Specify a soft id to earn 10% commission with your app.
        // Get yours at https://anti-captcha.com/clients/tools/devcenter
        .with_soft_id(0);
        // .quiet() turns the debug output off

    // Make sure the API key funds balance is positive
    let balance = ac.get_balance().await?;
    if balance <= 0.0 {
        // Stop here to make sure you don't DDoS the API while having empty balance
        eprintln!("Empty balance");
        return Ok(());
    }
    println!("Balance: {balance}");

    let solution = ac
        .solve_image_file("captcha.jpg", &ImageSettings {
            // Optional settings, see https://anti-captcha.com/apidoc/task-types/ImageToTextTask
            // phrase: true,               // the image has 2 or more words
            // case_sensitive: true,       // the answer is case sensitive
            // numeric: 1,                 // 1 - digits only, 2 - no digits
            // math_operation: true,       // the answer is the result of 50+5
            // min_length: 1,
            // max_length: 10,
            language_pool: "en".into(),    // "en" or "rn"
            comment: "Type in green characters".into(),
            ..Default::default()
        })
        // OR ac.solve_image("image-encoded-in-base64", &settings)
        .await?;

    println!("Captcha Solution: {}", solution.text());

    // If the answer turns out to be wrong:
    // ac.report_incorrect_image_captcha(solution.task_id()).await?;

    Ok(())
}
```
&nbsp;

### Solve Recaptcha V2
```rust
use anticaptcha::{Client, RecaptchaV2};

let ac = Client::new("API_KEY_HERE");

let solution = ac.solve_recaptcha_v2(&RecaptchaV2 {
    website_url: "https://www.website.com/".into(),
    website_key: "6Lcyu8UZAAAAACwSh6Xf58WrNXTu0LLu4F85xf20".into(),
    is_invisible: false,         // set to true if you are solving an invisible Recaptcha V2
    data_s_value: String::new(), // fill this for a Recaptcha V2 with the "data-s" parameter,
                                 // typically found at google.com websites
    ..Default::default()
}).await?;

println!("Recaptcha g-response token: {}", solution.g_recaptcha_response());
// In case you need the worker's user-agent
println!("User-Agent: {}", solution.user_agent());
```
Also with [proxy](https://anti-captcha.com/apidoc/task-types/RecaptchaV2Task):
```rust
use anticaptcha::{Proxy, ProxyType, RecaptchaV2};

let solution = ac.solve_recaptcha_v2_proxy_on(&RecaptchaV2 {
    website_url: "https://www.website.com/".into(),
    website_key: "6Lcyu8UZAAAAACwSh6Xf58WrNXTu0LLu4F85xf20".into(),
    user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36".into(),
    proxy: Proxy::new(ProxyType::Http, "1.2.3.4", 1234)
        .with_auth("login-optional", "pass-optional"),
    ..Default::default()
}).await?;
```
&nbsp;

### Solve Recaptcha V2 Enterprise
The same struct, `is_enterprise` switches the task type:
```rust
use std::collections::BTreeMap;
use anticaptcha::RecaptchaV2;

let mut enterprise_payload = BTreeMap::new();
enterprise_payload.insert("s".to_owned(), "SOME_ADDITIONAL_TOKEN".to_owned());

let solution = ac.solve_recaptcha_v2(&RecaptchaV2 {
    website_url: "https://store.steampowered.com/join".into(),
    website_key: "6LdIFr0ZAAAAAO3vz0O0OQrtAefzdJcWQM2TMYQH".into(),
    is_enterprise: true,
    enterprise_payload,
    // api_domain: "recaptcha.net".into(),  // only for a non-google.com script domain
    ..Default::default()
}).await?;
```
Use `solve_recaptcha_v2_proxy_on` for the proxy-on version.

&nbsp;

### Solve Recaptcha V3
```rust
use anticaptcha::RecaptchaV3;

let solution = ac.solve_recaptcha_v3(&RecaptchaV3 {
    website_url: "https://www.website.com/".into(),
    website_key: "6LcvNcwdAAAAAMWAuNRXH74u3QePsEzTm6GEjx0J".into(),
    page_action: "somefun".into(),
    min_score: 0.9,          // one of 0.3, 0.7, 0.9
    // is_enterprise: true,  // set to true for a Recaptcha V3 Enterprise
    ..Default::default()
}).await?;

println!("Recaptcha g-response token: {}", solution.g_recaptcha_response());
```
Recaptcha V3 has no proxy-on version.

&nbsp;

### Solve Hcaptcha
```rust
use anticaptcha::HCaptcha;

let solution = ac.solve_hcaptcha(&HCaptcha {
    website_url: "https://www.website.com/".into(),
    website_key: "00000000-1111-2222-3333-444444444444".into(),
    // is_invisible: true,
    // is_enterprise: true,
    // hCaptcha Enterprise parameters like rqdata, sentry, apiEndpoint, endpoint,
    // reportapi, assethost, imghost:
    // enterprise_payload: [("rqdata".to_owned(), "value".to_owned())].into(),
    ..Default::default()
}).await?;

println!("Hcaptcha Token: {}", solution.g_recaptcha_response());
// Use this user-agent for the form submission
println!("User-Agent: {}", solution.user_agent());
// Optional "respkey" value, you may need it too
println!("respkey: {}", solution.resp_key());
```
Also with [proxy](https://anti-captcha.com/apidoc/task-types/HCaptchaTask) — `solve_hcaptcha_proxy_on` and a filled `proxy` field.

&nbsp;

### Solve FunCaptcha
```rust
use anticaptcha::FunCaptcha;

let solution = ac.solve_funcaptcha(&FunCaptcha {
    website_url: "https://www.website.com/".into(),
    website_public_key: "00000000-1111-2222-3333-444444444444".into(),
    // Make sure to find and set this correctly, look for a URL like
    // https://somewebsite-api.arkoselabs.com/v2/00000000-1111-2222-3333-444444444444/api.js
    api_subdomain: "somewebsite-api.arkoselabs.com".into(),
    data_blob: r#"{"blob":"HERE_COMES_THE_blob_VALUE"}"#.into(),
    ..Default::default()
}).await?;

println!("Funcaptcha Token: {}", solution.token());
```
Also with [proxy](https://anti-captcha.com/apidoc/task-types/FunCaptchaTask) — `solve_funcaptcha_proxy_on`.

&nbsp;

### Solve Turnstile
```rust
use anticaptcha::Turnstile;

let solution = ac.solve_turnstile(&Turnstile {
    website_url: "https://www.website.com/".into(),
    website_key: "0x4AAAAAAABD2Inoxs-yJ8bz".into(),
    // action: "optional page action".into(),
    // c_data: "cdata token for cloudflare".into(),
    // chl_page_data: "chlPageData token for cloudflare".into(),
    ..Default::default()
}).await?;

println!("Turnstile Token: {}", solution.token());
// In case you need the worker's user-agent
println!("User-Agent: {}", solution.user_agent());
```
Also with [proxy](https://anti-captcha.com/apidoc/task-types/TurnstileTask) — `solve_turnstile_proxy_on`.

&nbsp;

### Solve GeeTest
GeeTest has 2 versions, number 3 and 4. Number 3 requires the parameter `challenge`. Number 4 has the optional setting `init_parameters`.
```rust
use anticaptcha::GeeTest;
use serde_json::json;

let solution = ac.solve_geetest(&GeeTest {
    website_url: "https://bitget.com/".into(),
    gt: "e9ca9c9ca19ad540a8017f5c107b2d0f".into(),

    // Solve GeeTest 4:
    version: 4,
    init_parameters: json!({ "riskType": "slide" }).as_object().cloned().unwrap_or_default(),

    // Solve GeeTest 3:
    // version: 3,
    // challenge: "1234567890abcdef1234567890abcdef".into(),

    ..Default::default()
}).await?;

// GeeTest v4
println!("captcha_id: {}", solution.captcha_id());
println!("lot_number: {}", solution.lot_number());
println!("pass_token: {}", solution.pass_token());
println!("gen_time: {}", solution.gen_time());
println!("captcha_output: {}", solution.captcha_output());

// GeeTest v3
// println!("{} {} {}", solution.challenge(), solution.seccode(), solution.validate());
```
Also with [proxy](https://anti-captcha.com/apidoc/task-types/GeeTestTask) — `solve_geetest_proxy_on`.

&nbsp;

### Image to coordinates
```rust
use anticaptcha::ImageToCoordinates;

let solution = ac.solve_image_to_coordinates_file("coordinates.jpg", &ImageToCoordinates {
    mode: "points".into(),   // "points" or "rectangles"
    comment: "Select objects in the specified order".into(),
    ..Default::default()
}).await?;
// OR ac.solve_image_to_coordinates("image-encoded-in-base64", &settings)

println!("Objects X,Y coordinates: {:?}", solution.coordinates());
```
&nbsp;

### Solve AntiGate (custom tasks)
```rust
use anticaptcha::{AntiGate, Proxy, ProxyType};
use serde_json::json;

let solution = ac.solve_antigate(&AntiGate {
    website_url: "http://antigate.com/logintest.php".into(),
    template_name: "Sign-in and wait for control text".into(),
    variables: json!({
        "login_input_css": "#login",
        "login_input_value": "the login",
        "password_input_css": "#password",
        "password_input_value": "the password",
        "control_text": "You have been logged successfully",
    }).as_object().cloned().unwrap_or_default(),

    // The proxy is optional for AntiGate tasks
    use_proxy: true,
    proxy: Proxy::new(ProxyType::Http, "1.2.3.4", 1234)
        .with_auth("login-optional", "pass-optional"),
    // domains_of_interest: vec!["some-other-domain.com".into()],
    ..Default::default()
}).await?;

println!("cookies: {:?}", solution.cookies());
println!("localStorage: {:?}", solution.local_storage());
println!("fingerprint: {:?}", solution.fingerprint());
println!("url: {}", solution.url());
```
&nbsp;

### Get AntiBot cookies
Makes a worker open the page through your proxy and hands you back the anti-bot cookies, so you can reuse them in your own requests. The proxy is required — the cookies are only valid for the IP address they were issued to.
```rust
use anticaptcha::{AntiBotCookie, Proxy, ProxyType};

let solution = ac.solve_antibot_cookie(&AntiBotCookie {
    website_url: "https://www.somewebsite.com/".into(),
    proxy: Proxy::new(ProxyType::Http, "1.2.3.4", 3128).with_auth("login", "password"),
}).await?;

// Ready to be sent as a Cookie header
println!("Cookie: {}", solution.cookie_header());
println!("User-Agent: {}", solution.fingerprint_user_agent());
```
&nbsp;

### Solve Prosopo
```rust
use anticaptcha::Prosopo;

let solution = ac.solve_prosopo(&Prosopo {
    website_url: "https://www.website.com/".into(),
    website_key: "sitekey-here".into(),
    ..Default::default()
}).await?;

println!("Prosopo Token: {}", solution.token());
```
Also with [proxy](https://anti-captcha.com/apidoc/task-types/ProsopoTask) — `solve_prosopo_proxy_on`.

&nbsp;

### Solve Friendly Captcha
```rust
use anticaptcha::FriendlyCaptcha;

let solution = ac.solve_friendly_captcha(&FriendlyCaptcha {
    website_url: "https://www.website.com/".into(),
    website_key: "sitekey-here".into(),
    ..Default::default()
}).await?;

println!("Friendly Captcha Token: {}", solution.token());
```
Also with [proxy](https://anti-captcha.com/apidoc/task-types/FriendlyCaptchaTask) — `solve_friendly_captcha_proxy_on`.

&nbsp;

### Solve Amazon WAF
Two options here:

1. When the captcha is at the bot filtering page and you need the `aws-waf-token` cookie:
```rust
use anticaptcha::Amazon;

let solution = ac.solve_amazon(&Amazon {
    website_url: "https://www.website.com/".into(),
    website_key: "key_value_from_window.gokuProps_object".into(),
    iv: "iv_value_from_window.gokuProps_object".into(),
    context: "context_value_from_window.gokuProps_object".into(),
    // captcha_script: "optional_captcha.js_script_url".into(),
    // challenge_script: "optional_challenge.js_script_url".into(),
    ..Default::default()
}).await?;

println!("aws-waf-token: {}", solution.token());
```

2. When the captcha is a standalone widget triggered by a user's action:
```rust
let solution = ac.solve_amazon(&Amazon {
    website_url: "https://www.website.com/".into(),
    // Captcha widget's API key from the AwsWafCaptcha.renderCaptcha function
    website_key: "captcha_key_value".into(),
    waf_type: "widget".into(),
    // Full URL to jsapi.js
    jsapi_script: "https://164cb210e333.edge.captcha-sdk.awswaf.com/164cb210e333/jsapi.js".into(),
    ..Default::default()
}).await?;
```
Both options have a [proxy-on](https://anti-captcha.com/apidoc/task-types/AmazonTask) version — `solve_amazon_proxy_on`.

&nbsp;

### Solve Altcha
```rust
use anticaptcha::Altcha;

let solution = ac.solve_altcha(&Altcha {
    website_url: "https://www.website.com/".into(),

    // Option 1: use the challenge URL (use one of the options!)
    challenge_url: "/some/path/to/challenge/url".into(),

    // Option 2: use the challenge JSON
    // challenge_json: r#"{"algorithm":"SHA-256","challenge":"1a40f7ba3393f9513016879de41c7221f14e563856de2f647233a00accf9c28b","salt":"0887f273d79df143355b9e5f","signature":"1de2bbf282420aef6ca0a84c38c85e2b1e40023d28bef72278d735555a8f47fb"}"#.into(),

    ..Default::default()
}).await?;

println!("Altcha Token: {}", solution.token());
```
Also with [proxy](https://anti-captcha.com/apidoc/task-types/AltchaTask) — `solve_altcha_proxy_on`.

&nbsp;

### Synchronous API
Turn on the `blocking` feature:
```toml
[dependencies]
anticaptchaofficial = { version = "1", features = ["blocking"] }
```
```rust
use anticaptcha::blocking::Client;
use anticaptcha::RecaptchaV2;

fn main() -> anticaptcha::Result<()> {
    let ac = Client::new("API_KEY_HERE")?;

    let solution = ac.solve_recaptcha_v2(&RecaptchaV2 {
        website_url: "https://www.website.com/".into(),
        website_key: "6Lcyu8UZAAAAACwSh6Xf58WrNXTu0LLu4F85xf20".into(),
        ..Default::default()
    })?;

    println!("{}", solution.g_recaptcha_response());
    Ok(())
}
```
Every method has the same name and arguments as its async counterpart. The blocking client drives its own Tokio runtime, so do not call it from inside an async runtime — use the async `Client` there.

### Error handling
Every method returns `anticaptcha::Result<T>`:

| Variant | When |
|---|---|
| `Error::InvalidTask` | a required parameter is missing or out of range, nothing was sent |
| `Error::Network` | the API could not be reached |
| `Error::BadResponse` | the API answered with something unexpected |
| `Error::Api` | the API answered with a non-zero `errorId` |
| `Error::Timeout` | the task was still unsolved when the waiting limit ran out |
| `Error::Io` | a captcha image file could not be read |

```rust
match ac.solve_recaptcha_v2(&params).await {
    Ok(solution) => println!("{}", solution.g_recaptcha_response()),
    Err(error) if error.is_retryable() => println!("try again: {error}"),
    // https://anti-captcha.com/apidoc/errors
    Err(error) => eprintln!("{} {error}", error.api_error_code().unwrap_or("")),
}
```

### Reading the solution
`Solution` wraps the API's `solution` object. Named accessors return an empty string when the field is not there, so nothing panics on a task type that does not fill them:

```rust
solution.text();                  // image captchas
solution.token();                 // FunCaptcha, Turnstile, Prosopo, Friendly Captcha, Altcha, Amazon
solution.g_recaptcha_response();  // Recaptcha, hCaptcha
solution.user_agent();            // worker's user-agent
solution.cookies();               // AntiGate, AntiBotCookie -> Option<&serde_json::Value>
solution.coordinates();           // ImageToCoordinates -> Option<&serde_json::Value>
solution.task_id();               // for the report_* methods
solution.cost();                  // what the task cost, in US dollars
solution.raw();                   // the whole object, for anything not listed above
solution.get("someNewField");     // any string field by its API name
```

### Other settings
```rust
use std::time::Duration;

let ac = Client::new("API_KEY_HERE")
    .with_soft_id(1187)                          // earn 10% commission with your app
    .quiet()                                     // same as .with_verbose(false)
    .with_connection_timeout(Duration::from_secs(30))
    .with_polling_intervals(Duration::from_secs(5), Duration::from_secs(5))
    .with_max_waiting_time(Duration::from_secs(300))
    .with_http_client(my_reqwest_client);        // reuse your own connection pool
```

A task type this crate does not wrap yet can still be sent by hand:
```rust
use serde_json::json;

let solution = ac.solve_task(json!({
    "type": "SomeNewTaskProxyless",
    "websiteURL": "https://www.website.com/",
})).await?;
```

### Feature flags
| Feature | Meaning |
|---|---|
| `rustls-tls` *(default)* | TLS through rustls, no system OpenSSL needed |
| `native-tls` | TLS through the platform library |
| `blocking` | adds the synchronous `anticaptcha::blocking` module |

### Running the examples and tests
```bash
git clone https://github.com/anti-captcha/anticaptcha-rust.git
cd anticaptcha-rust

# unit tests, they need no API key and no network
cargo test --all-features

# put your API key into examples/solve.rs first
cargo run --example solve -- image
```
Run the example without an argument to see the full list.
