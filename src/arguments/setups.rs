use glob::glob;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::exit,
};

use crate::{files, prompt};

use super::SCAN_FILE_TYPE;

pub fn get_todo(project_folder: &PathBuf) -> Option<HashMap<String, HashMap<usize, String>>> {
    let mut file_todo = HashMap::new();

    for files in glob(project_folder.join("*").to_str().unwrap()).unwrap() {
        if let Ok(file) = files {
            if !file.is_file() {
                continue;
            }

            match file.extension() {
                Some(ext) => {
                    if !SCAN_FILE_TYPE.contains(&ext.to_str().unwrap()) {
                        continue;
                    }
                }
                None => continue,
            }

            let lines: Vec<String> = fs::read_to_string(&file)
                .unwrap()
                .lines()
                .map(String::from)
                .collect();

            let mut todo_dict = HashMap::new();
            lines.iter().enumerate().rev().for_each(|(idx, line)| {
                if line.contains("TODO") {
                    todo_dict.insert(idx + 1, line.to_string());
                }
            });

            if !todo_dict.is_empty() {
                file_todo.insert(
                    file.file_name().unwrap().to_str().unwrap().to_string(),
                    todo_dict,
                );
            }
        }
    }

    if file_todo.len() != 0 {
        return Some(file_todo);
    }

    None
}

pub fn verify_logic() {
    let misplaced = files::get_misplaced_nodes();
    if misplaced.len() != 0 {
        println!("These directories are not in their recommended locations:");
        for (local_directory, valid_directory) in &misplaced {
            println!(
                "  {} -> {}",
                local_directory.to_str().unwrap().to_string(),
                valid_directory.to_str().unwrap().to_string()
            );
        }

        if !prompt::yes_no("Would you like to move them?") {
            return;
        }

        // If you want to replace them
        for (local_directory, valid_directory) in &misplaced {
            if let Some(parent) = valid_directory.parent() {
                if !Path::exists(parent) {
                    if let Err(err) = fs::create_dir(parent) {
                        eprintln!("Failed to create new node directory: {}", err);
                    }
                }
            }

            if let Err(err) = fs::rename(local_directory, valid_directory) {
                println!(
                    "{} -> {}",
                    local_directory.to_str().unwrap(),
                    valid_directory.to_str().unwrap()
                );
                println!("Can't move folder because: {}", err);
                exit(1);
            }
        }
    }

    if let Some(empty_dirs) = files::get_empty_lms() {
        println!("\nThe following folders are empty");
        for dir in &empty_dirs {
            println!("  - {}", dir.to_str().unwrap());
        }

        if prompt::yes_no("\nDo you want to remove them") {
            for dir in &empty_dirs {
                if let Err(err) = fs::remove_dir(dir) {
                    eprintln!("Can't remove folder: {}", err);
                    exit(1)
                }
            }
        }
    }

    println!("All nodes are in the right place!");
}