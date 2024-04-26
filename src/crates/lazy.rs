use std::env;
use std::sync::{Arc, Mutex, Once};

static INIT: Once = Once::new();
static mut BASE_URL: Option<Arc<Mutex<String>>> = None;

fn initialize_base_url() -> Arc<Mutex<String>> {
    let url = env::var("LMS_BASE_URL").unwrap_or("https://sd42.nl".to_string());
    Arc::new(Mutex::new(url))
}

pub struct LazyBaseUrl;

impl LazyBaseUrl {
    fn get_base_url() -> Arc<Mutex<String>> {
        unsafe {
            INIT.call_once(|| {
                let url = initialize_base_url();
                BASE_URL = Some(url);
            });
            BASE_URL.as_ref().expect("BASE_URL not initialized").clone()
        }
    }
}

impl std::fmt::Display for LazyBaseUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let url = LazyBaseUrl::get_base_url();
        let locked_url = url.lock().unwrap();
        write!(f, "{}", *locked_url)
    }
}
