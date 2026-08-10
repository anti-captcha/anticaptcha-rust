//! Runnable examples for every task type.
//!
//! Put your API key below and run:
//!
//! ```bash
//! cargo run --example solve -- image
//! ```
//!
//! Run without an argument to see the list of available examples.

use std::collections::BTreeMap;

use anticaptcha::{
    Altcha, Amazon, AntiBotCookie, AntiGate, Client, FriendlyCaptcha, FunCaptcha, GeeTest,
    HCaptcha, ImageSettings, ImageToCoordinates, Prosopo, Proxy, ProxyType, RecaptchaV2,
    RecaptchaV3, Result, Turnstile,
};
use serde_json::json;

const API_KEY: &str = "API_KEY_HERE";

/// Specify a soft id to earn 10% commission with your app.
/// Get yours at https://anti-captcha.com/clients/tools/devcenter
const SOFT_ID: i64 = 0;

const EXAMPLES: &[&str] = &[
    "balance",
    "image",
    "coordinates",
    "recaptcha2",
    "recaptcha2-proxy",
    "recaptcha2-enterprise",
    "recaptcha3",
    "hcaptcha",
    "funcaptcha",
    "geetest3",
    "geetest4",
    "turnstile",
    "prosopo",
    "friendly",
    "amazon",
    "altcha",
    "antigate",
    "antibot-cookie",
];

fn client() -> Client {
    Client::new(API_KEY).with_soft_id(SOFT_ID)
    // .quiet() turns the debug output off
}

/// Every proxy-on example uses this. Do not use purchased or rented proxies from
/// proxy services, use proper proxy software like Squid.
fn sample_proxy() -> Proxy {
    Proxy::new(ProxyType::Http, "1.2.3.4", 1234).with_auth("login-optional", "password-optional")
}

#[tokio::main]
async fn main() {
    let name = std::env::args().nth(1).unwrap_or_default();

    if !EXAMPLES.contains(&name.as_str()) {
        println!("Usage: cargo run --example solve -- <example>");
        println!("Available examples: {}", EXAMPLES.join(", "));
        std::process::exit(1);
    }

    if let Err(error) = run(&name).await {
        eprintln!("Failed: {error}");

        if let Some(code) = error.api_error_code() {
            eprintln!("See https://anti-captcha.com/apidoc/errors for {code}");
        }

        std::process::exit(1);
    }
}

async fn run(name: &str) -> Result<()> {
    match name {
        "balance" => balance().await,
        "image" => image().await,
        "coordinates" => coordinates().await,
        "recaptcha2" => recaptcha_v2().await,
        "recaptcha2-proxy" => recaptcha_v2_proxy().await,
        "recaptcha2-enterprise" => recaptcha_v2_enterprise().await,
        "recaptcha3" => recaptcha_v3().await,
        "hcaptcha" => hcaptcha().await,
        "funcaptcha" => funcaptcha().await,
        "geetest3" => geetest_v3().await,
        "geetest4" => geetest_v4().await,
        "turnstile" => turnstile().await,
        "prosopo" => prosopo().await,
        "friendly" => friendly_captcha().await,
        "amazon" => amazon().await,
        "altcha" => altcha().await,
        "antigate" => antigate().await,
        "antibot-cookie" => antibot_cookie().await,
        _ => unreachable!("the name was checked in main"),
    }
}

async fn balance() -> Result<()> {
    let ac = client();

    println!("Balance: {}", ac.get_balance().await?);
    println!("Captcha credits: {}", ac.get_credits_balance().await?);

    Ok(())
}

async fn image() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_image_file(
            "examples/captcha.jpg",
            &ImageSettings {
                language_pool: "en".into(), // "en" or "rn"
                comment: "Type in green characters".into(),
                // phrase: true,           // the image has 2 or more words
                // case_sensitive: true,   // the answer is case sensitive
                // numeric: 1,             // 1 - digits only, 2 - no digits
                // math_operation: true,   // the answer is the result of 50+5
                // min_length: 1,
                // max_length: 10,
                ..Default::default()
            },
        )
        .await?;

    println!("Captcha text: {}", solution.text());

    // If the answer turns out to be wrong:
    // ac.report_incorrect_image_captcha(solution.task_id()).await?;

    Ok(())
}

async fn coordinates() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_image_to_coordinates_file(
            "examples/coordinates.jpg",
            &ImageToCoordinates {
                mode: "points".into(), // "points" or "rectangles"
                comment: "Select objects in the specified order".into(),
                ..Default::default()
            },
        )
        .await?;

    println!("Objects X,Y coordinates: {:?}", solution.coordinates());

    Ok(())
}

async fn recaptcha_v2() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_recaptcha_v2(&RecaptchaV2 {
            website_url: "https://www.website.com/".into(),
            website_key: "6Lcyu8UZAAAAACwSh6Xf58WrNXTu0LLu4F85xf20".into(),
            // is_invisible: true,             // solving an invisible Recaptcha V2
            // data_s_value: "...".into(),     // the "data-s" parameter, typical for google.com
            ..Default::default()
        })
        .await?;

    println!("g-response token: {}", solution.g_recaptcha_response());
    println!("Worker's user-agent: {}", solution.user_agent());

    Ok(())
}

async fn recaptcha_v2_proxy() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_recaptcha_v2_proxy_on(&RecaptchaV2 {
            website_url: "https://www.website.com/".into(),
            website_key: "6Lcyu8UZAAAAACwSh6Xf58WrNXTu0LLu4F85xf20".into(),
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                         (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"
                .into(),
            proxy: sample_proxy(),
            ..Default::default()
        })
        .await?;

    println!("g-response token: {}", solution.g_recaptcha_response());

    Ok(())
}

async fn recaptcha_v2_enterprise() -> Result<()> {
    let ac = client();

    let mut enterprise_payload = BTreeMap::new();
    enterprise_payload.insert("s".to_owned(), "SOME_ADDITIONAL_TOKEN".to_owned());

    let solution = ac
        .solve_recaptcha_v2(&RecaptchaV2 {
            website_url: "https://store.steampowered.com/join".into(),
            website_key: "6LdIFr0ZAAAAAO3vz0O0OQrtAefzdJcWQM2TMYQH".into(),
            is_enterprise: true,
            enterprise_payload,
            // api_domain: "recaptcha.net".into(),  // only for a non-google.com script domain
            ..Default::default()
        })
        .await?;

    println!("g-response token: {}", solution.g_recaptcha_response());

    Ok(())
}

async fn recaptcha_v3() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_recaptcha_v3(&RecaptchaV3 {
            website_url: "https://www.website.com/".into(),
            website_key: "6LcvNcwdAAAAAMWAuNRXH74u3QePsEzTm6GEjx0J".into(),
            page_action: "somefun".into(),
            min_score: 0.9, // one of 0.3, 0.7, 0.9
            // is_enterprise: true,
            ..Default::default()
        })
        .await?;

    println!("g-response token: {}", solution.g_recaptcha_response());

    Ok(())
}

async fn hcaptcha() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_hcaptcha(&HCaptcha {
            website_url: "https://www.website.com/".into(),
            website_key: "00000000-1111-2222-3333-444444444444".into(),
            // is_invisible: true,
            // is_enterprise: true,
            // enterprise_payload: [("rqdata".to_owned(), "value".to_owned())].into(),
            ..Default::default()
        })
        .await?;

    println!("hCaptcha token: {}", solution.g_recaptcha_response());
    println!(
        "Use this user-agent for the form: {}",
        solution.user_agent()
    );
    println!("respkey: {}", solution.resp_key());

    Ok(())
}

async fn funcaptcha() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_funcaptcha(&FunCaptcha {
            website_url: "https://www.website.com/".into(),
            website_public_key: "00000000-1111-2222-3333-444444444444".into(),
            // Look for a URL like
            // https://somewebsite-api.arkoselabs.com/v2/00000000-1111-2222-3333-444444444444/api.js
            api_subdomain: "somewebsite-api.arkoselabs.com".into(),
            data_blob: r#"{"blob":"HERE_COMES_THE_blob_VALUE"}"#.into(),
            ..Default::default()
        })
        .await?;

    println!("FunCaptcha token: {}", solution.token());

    Ok(())
}

async fn geetest_v3() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_geetest(&GeeTest {
            website_url: "https://www.website.com/".into(),
            gt: "b6e21f90a91a3c2d4a31fe84e10d0442".into(),
            // The challenge is one-time, grab a fresh one for every task
            challenge: "169acd4a58f2c99770322dfa5270c221".into(),
            version: 3,
            ..Default::default()
        })
        .await?;

    println!("challenge: {}", solution.challenge());
    println!("seccode: {}", solution.seccode());
    println!("validate: {}", solution.validate());

    Ok(())
}

async fn geetest_v4() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_geetest(&GeeTest {
            website_url: "https://www.website.com/".into(),
            gt: "e9ca9c9ca19ad540a8017f5c107b2d0f".into(),
            version: 4,
            init_parameters: json!({ "riskType": "slide" })
                .as_object()
                .cloned()
                .unwrap_or_default(),
            ..Default::default()
        })
        .await?;

    println!("captcha_id: {}", solution.captcha_id());
    println!("lot_number: {}", solution.lot_number());
    println!("pass_token: {}", solution.pass_token());
    println!("gen_time: {}", solution.gen_time());
    println!("captcha_output: {}", solution.captcha_output());

    Ok(())
}

async fn turnstile() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_turnstile(&Turnstile {
            website_url: "https://www.website.com/".into(),
            website_key: "0x4AAAAAAABD2Inoxs-yJ8bz".into(),
            // action: "optional page action".into(),
            // c_data: "cdata token for cloudflare".into(),
            // chl_page_data: "chlPageData token for cloudflare".into(),
            ..Default::default()
        })
        .await?;

    println!("Turnstile token: {}", solution.token());

    Ok(())
}

async fn prosopo() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_prosopo(&Prosopo {
            website_url: "https://www.website.com/".into(),
            website_key: "sitekey-here".into(),
            ..Default::default()
        })
        .await?;

    println!("Prosopo token: {}", solution.token());

    Ok(())
}

async fn friendly_captcha() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_friendly_captcha(&FriendlyCaptcha {
            website_url: "https://www.website.com/".into(),
            website_key: "sitekey-here".into(),
            ..Default::default()
        })
        .await?;

    println!("Friendly Captcha token: {}", solution.token());

    Ok(())
}

async fn amazon() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_amazon(&Amazon {
            website_url: "https://www.website.com/".into(),
            website_key: "key_value_from_window.gokuProps_object".into(),
            iv: "iv_value_from_window.gokuProps_object".into(),
            context: "context_value_from_window.gokuProps_object".into(),
            // For a standalone widget instead of the bot filtering page:
            // waf_type: "widget".into(),
            // jsapi_script: "https://164cb210e333.edge.captcha-sdk.awswaf.com/164cb210e333/jsapi.js".into(),
            ..Default::default()
        })
        .await?;

    println!("aws-waf-token: {}", solution.token());

    Ok(())
}

async fn altcha() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_altcha(&Altcha {
            website_url: "https://www.website.com/".into(),
            // Use challenge_url or challenge_json, not both
            challenge_url: "/some/path/to/challenge/url".into(),
            // challenge_json: r#"{"algorithm":"SHA-256","challenge":"..."}"#.into(),
            ..Default::default()
        })
        .await?;

    println!("Altcha token: {}", solution.token());

    Ok(())
}

async fn antigate() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_antigate(&AntiGate {
            website_url: "http://antigate.com/logintest.php".into(),
            template_name: "Sign-in and wait for control text".into(),
            variables: json!({
                "login_input_css": "#login",
                "login_input_value": "the login",
                "password_input_css": "#password",
                "password_input_value": "the password",
                "control_text": "You have been logged successfully",
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
            // The proxy is optional for AntiGate tasks
            // use_proxy: true,
            // proxy: sample_proxy(),
            ..Default::default()
        })
        .await?;

    println!("cookies: {:?}", solution.cookies());
    println!("localStorage: {:?}", solution.local_storage());
    println!("fingerprint: {:?}", solution.fingerprint());
    println!("url: {}", solution.url());

    Ok(())
}

async fn antibot_cookie() -> Result<()> {
    let ac = client();

    let solution = ac
        .solve_antibot_cookie(&AntiBotCookie {
            website_url: "https://www.somewebsite.com/".into(),
            // The cookies are bound to this IP address, use the very same proxy afterwards
            proxy: sample_proxy(),
        })
        .await?;

    println!(
        "Use these cookies for requests: {}",
        solution.cookie_header()
    );
    println!(
        "Use this user-agent for requests: {}",
        solution.fingerprint_user_agent()
    );

    Ok(())
}
