use configparser::ini::Ini;
use std::env;
use std::path::{Path, PathBuf};

const FALLBACK: [&str; 4] = ["nvim", "code", "vscode", "codium"];

pub struct Settings {
    pub config: Ini,
    config_path: PathBuf,
    pub editors: Vec<String>,
}

impl Settings {
    pub fn new() -> Self {
        let mut config = Ini::new();
        let mut config_path = PathBuf::new();

        config_path.push(env::var("HOME").unwrap());
        config_path.push(".config");
        config_path.push("lms.ini");

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

    pub fn get_setting(&self, section: &str, key: &str, default: bool) -> bool {
        self.config
            .getbool(section, key)
            .unwrap()
            .unwrap_or(default)
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
