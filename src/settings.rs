use configparser::ini::{Ini, WriteOptions};
use std::env;
use std::path::{Path, PathBuf};

const FALLBACK: [&str; 4] = ["nvim", "code", "vscode", "codium"];

pub struct Settings {
    config: Ini,
    config_path: PathBuf,
    pub editors: Vec<String>,
}

impl Settings {
    pub fn new() -> Self {
        let mut config = Ini::new();
        let mut config_path = PathBuf::new();

        config_path.push(env::var("HOME").unwrap());
        config_path.push(".config");

        if cfg!(test) {
            config_path.push("lms_test.ini");
        } else {
            config_path.push("lms.ini");
        }

        let mut editors: Vec<String> = Vec::new();

        if Path::exists(&config_path) {
            let map = config.load(&config_path).unwrap();

            if let Some(custom) = map.get("custom") {
                if let Some(editor_value) = custom.get("editor") {
                    editors.push(editor_value.clone().unwrap())
                }
            }
        }

        let editors = editors
            .iter()
            .map(|s| s.to_string())
            .chain(FALLBACK.iter().map(|&s| s.to_string()))
            .collect::<Vec<String>>();

        Self {
            config,
            config_path,
            editors,
        }
    }

    pub fn get_token(&self) -> String {
        self.config.get("auth", "token").unwrap()
    }

    pub fn get_bool(&self, section: &str, key: &str, default: bool) -> bool {
        self.config
            .getbool(section, key)
            .unwrap()
            .unwrap_or(default)
    }

    pub fn get_string(&self, section: &str, key: &str, default: String) -> String {
        self.config.get(section, key).unwrap_or(default)
    }

    pub fn pretty_print(&self) -> String {
        self.config.pretty_writes(&WriteOptions::default())
    }

    pub fn set(&mut self, category: String, name: String, value: String) {
        self.config.set(&category, &name, Some(value));
        if let Some(path_str) = self.config_path.to_str() {
            let _ = self.config.write(path_str);
        } else {
            panic!("No lms.ini found")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::create_test_settings;

    #[test]
    fn test_new_settings() {
        let settings = create_test_settings();
        assert_eq!(settings.editors, FALLBACK.to_vec());
    }

    #[test]
    fn test_get_token() {
        let settings = create_test_settings();
        assert_eq!(settings.get_token(), "token");
    }

    #[test]
    fn test_get_bool() {
        let settings = create_test_settings();
        assert_eq!(settings.get_bool("setup", "check_todo", false), true);
    }

    #[test]
    fn test_get_string() {
        let settings = create_test_settings();
        assert_eq!(
            settings.get_string("auth", "token", "default".to_string()),
            "token"
        );
    }

    #[test]
    fn test_pretty_print() {
        let settings = create_test_settings();
        // The order of the lines is not guaranteed, so we sort them
        let expected = "[auth]\ntoken=token\n[setup]\ncheck_todo=true\n"
            .lines()
            .collect::<Vec<_>>()
            .sort();

        let binding = settings.pretty_print();
        let actual = binding.lines().collect::<Vec<_>>().sort();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_set_string() {
        let mut settings = create_test_settings();
        settings.set(
            "auth".to_string(),
            "token".to_string(),
            "new_token".to_string(),
        );
        assert_eq!(
            settings.get_string("auth", "token", "default".to_string()),
            "new_token"
        );
    }

    #[test]
    fn test_set_bool() {
        let mut settings = create_test_settings();
        settings.set(
            "setup".to_string(),
            "check_todo".to_string(),
            "false".to_string(),
        );
        assert_eq!(settings.get_bool("setup", "check_todo", false), false);
    }

    #[test]
    fn test_set_default() {
        let settings = create_test_settings();
        assert_eq!(
            settings.get_string("setup", "new_key", "default".to_string()),
            "default"
        );
    }
}
