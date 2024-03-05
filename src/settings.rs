use std::fs;
use std::path::{Path, PathBuf};
use std::env;
use configparser::ini::Ini;

const FALLBACK: [&str; 4] = ["nvim .", "vscode .", "codium .", "nvim ."];

pub struct Settings {
    pub config: Ini,
    pub token: String,
    pub editors: Vec<String>,
    pub setup: bool,
    pub move_node_directories: bool
}


impl Settings {
    pub fn new() -> Self {

        let mut config = Ini::new();
        let mut config_location = PathBuf::new();

        config_location.push(env::var("HOME").unwrap());
        config_location.push(".config");
        config_location.push("lms.ini");

        let mut editors: Vec<String> = Vec::new();

        if Path::exists(&config_location) {
            // {auth: {token: 1234}, setup: {enabled: true}, custom: {editor: nvim}}
            let map = config.load(&config_location);
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
            token: "".to_string(),
            editors,
            setup: true,
            move_node_directories: true
        }
    }
}
