use std::fs;
use std::path::{Path, PathBuf};
use std::env;
use configparser::ini::Ini;

const FALLBACK: [&str; 4] = ["nvim .", "vscode .", "codium .", "nvim ."];

pub struct Settings {
    config: Ini,
    config_path: PathBuf,
    pub token: String,
    pub editors: Vec<String>,
    pub setup: bool,
    pub move_node_directories: bool
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
            // {auth: {token: 1234}, setup: {enabled: true}, custom: {editor: nvim}}
            let map = config.load(&config_path);
            let token = "";
            let setup = "";
            let custom_editor = "";

            editors.push(custom_editor.to_string());

        } 

        let editors = editors
            .iter()
            .map(|s| s.to_string())
            .chain(FALLBACK.iter().map(|&s| s.to_string()))
            .collect::<Vec<String>>();
        Self {
            config,
            config_path,
            token: "".to_string(),
            editors,
            setup: true,
            move_node_directories: true
        }
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
