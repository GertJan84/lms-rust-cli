use glob::glob;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::exit,
};

use crate::{error_exit, files, prompt, stru, ustr_ustring, ustring};

use super::SCAN_FILE_TYPE;

// TODO: Move function to different location
pub fn get_attempt_files_content(
    project_folder: &PathBuf,
) -> Option<HashMap<PathBuf, Vec<String>>> {
    let mut files_content: HashMap<PathBuf, Vec<String>> = HashMap::new();

    for files in glob(stru!(project_folder.join("*"))).unwrap() {
        if let Ok(file) = files {
            if !file.is_file() {
                continue;
            }

            match file.extension() {
                Some(ext) => {
                    if !SCAN_FILE_TYPE.contains(&stru!(ext)) {
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

            files_content.insert(file, lines);
        }
    }

    if !files_content.is_empty() {
        return Some(files_content);
    }

    None
}

pub fn get_todo(project_folder: &PathBuf) -> Option<HashMap<String, HashMap<usize, String>>> {
    let mut file_todo = HashMap::new();

    if let Some(found_files) = get_attempt_files_content(&project_folder) {
        for (file_location, content) in found_files {
            let mut todo_dict = HashMap::new();
            content.iter().enumerate().rev().for_each(|(idx, line)| {
                if line.contains("TODO") {
                    todo_dict.insert(idx + 1, line.to_string());
                }
            });

            if !todo_dict.is_empty() {
                file_todo.insert(ustr_ustring!(file_location.file_name()), todo_dict);
            }
        }
    }

    if !file_todo.is_empty() {
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
                ustring!(local_directory.to_str()),
                ustring!(valid_directory.to_str())
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
                println!("{} -> {}", stru!(local_directory), stru!(valid_directory));
                error_exit!("Can't move folder because: {}", err);
            }
        }
    }

    if let Some(empty_dirs) = files::get_empty_lms() {
        println!("\nThe following folders are empty");
        for dir in &empty_dirs {
            println!("  - {}", stru!(dir));
        }

        if prompt::yes_no("\nDo you want to remove them") {
            for dir in &empty_dirs {
                if let Err(err) = fs::remove_dir(dir) {
                    error_exit!("Can't remove folder: {}", err);
                }
            }
        }
    }

    println!("All nodes are in the right place!");
}
