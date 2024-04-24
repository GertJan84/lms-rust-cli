use gethostname::gethostname;
use rand::{distributions::Alphanumeric, Rng};
use url::form_urlencoded;

use crate::{arguments::AUTH_TOKEN_LENGTH, settings::Settings};

pub fn login_logic(mut settings: Settings) {
    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(AUTH_TOKEN_LENGTH.into())
        .map(char::from)
        .collect();

    settings.set("auth".to_string(), "token".to_string(), token.clone());
    let encoded_host =
        form_urlencoded::byte_serialize(gethostname().as_encoded_bytes()).collect::<String>();
    let url = format!(
        "{}/api/authorize?host={}&token={}",
        crate::BASE_URL.to_string(),
        encoded_host,
        &token
    );
    println!("Go to this URL to authorize lms: {}", url);
    let _ = webbrowser::open(url.as_str());
}
