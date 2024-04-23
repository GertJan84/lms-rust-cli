use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{exit, Command},
};

use crate::{io, settings::Settings};


fn is_hidden(path: &Path) -> bool {
    // Check if the file or directory name starts with a dot (".")
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}


pub fn open_ide(path: &PathBuf, editors: &Vec<String>) -> () {
    let settings = Settings::new();

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


    if settings.get_bool("setup", "open_first_folder", false) {
        if let Ok(entries) = fs::read_dir(&path) {
            let mut unhidden_dirs = Vec::with_capacity(4);
            for entry in entries.flatten() {
                let entry_path = entry.path();
                
                if !entry_path.is_dir() && is_hidden(&entry_path) {
                    continue;
                }
    
                unhidden_dirs.push(entry_path)
    
            }
    
            if unhidden_dirs.len() == 1 {
                let _ = env::set_current_dir(unhidden_dirs.first().unwrap());
            }
    
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
