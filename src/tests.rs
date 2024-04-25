use crate::settings::Settings;

/// Create a new Settings object with some test values
/// Dead code because it's only used in tests
#[warn(dead_code)]
pub fn create_test_settings() -> Settings {
    let mut settings = Settings::new();
    settings.set("auth".to_string(), "token".to_string(), "token".to_string());
    settings.set(
        "setup".to_string(),
        "check_todo".to_string(),
        "true".to_string(),
    );
    settings
}
