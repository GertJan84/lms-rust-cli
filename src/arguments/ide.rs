use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{exit, Command},
};

use crate::io;

pub fn open_ide(path: &PathBuf, editors: &Vec<String>) -> () {
    if let Err(err) = env::set_current_dir(&path) {
        eprintln!("{}", err);
        return;
    }

    let mut editors = editors.clone();

    if Path::new(".lms-ide").exists() {
        match fs::read_to_string(".lms-ide") {
            Ok(lms_ide) => {
                // Parse lms_ide file to exclude dots and remove white so that "android-studio . " becomes "android-studio"
                let lms_ide = lms_ide
                    .split_whitespace()
                    .filter(|&x| !x.contains("."))
                    .collect();

                editors.insert(0, lms_ide)
            }
            Err(_) => {}
        }
    }

    for editor in &editors {
        let mut editor_parts = editor.split_whitespace();
        let editor_name = editor_parts.next().unwrap_or_default();
        let mut args: Vec<&str> = editor_parts.collect();

        if args.is_empty() {
            args.push(".");
        }

        // Skip to the next editor if the current one is not available
        if !io::is_installed(editor_name) {
            continue;
        }

        Command::new(editor_name)
            .args(args)
            .status()
            .expect("Failed to execute editor");
        exit(0);
    }
}
