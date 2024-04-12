use crate::io;
use glob::glob;
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    process::exit,
};

pub fn get_lms_dir() -> PathBuf {
    let mut lms_dir = PathBuf::new();
    lms_dir.push(env::var("HOME").unwrap());
    lms_dir.push("lms");

    lms_dir
}

pub fn is_folder_empty(path: &PathBuf) -> bool {
    if !Path::exists(&path) {
        return false;
    }

    if let Ok(dir_entries) = fs::read_dir(path) {
        for _ in dir_entries {
            return false;
        }
        return true;
    }

    false
}

pub fn get_empty_lms() -> Option<HashSet<PathBuf>> {
    let lms_dir = get_lms_dir().join("*");
    let mut empty_dirs: HashSet<PathBuf> = HashSet::new();

    for dir in glob(lms_dir.to_str().unwrap()).expect("Failed to read lms dir") {
        if let Ok(path) = dir {
            if !path.is_dir() {
                continue;
            }

            if !is_folder_empty(&path) {
                continue;
            }

            empty_dirs.insert(path);
        }
    }

    if empty_dirs.is_empty() {
        None
    } else {
        Some(empty_dirs)
    }
}

pub fn get_misplaced_nodes() -> HashMap<PathBuf, PathBuf> {
    let lms_dir = get_lms_dir();

    let correct_paths_json =
        match io::request("GET", "/api/node-paths".to_string(), &"".to_string(), None) {
            Some(data) => io::response_to_json(data),
            None => {
                eprintln!("Cant convert paths to json");
                exit(1)
            }
        };

    let mut misplaced: HashMap<PathBuf, PathBuf> = HashMap::new();

    let target_dir = lms_dir.join("*/*");
    // Get all directories in lms [python, pwa, static-web, ..etc]

    let correct_nodes = &correct_paths_json.as_array().unwrap()[0]
        .as_object()
        .unwrap();
    for dir in glob(target_dir.to_str().unwrap()).expect("Failed to read lms dir") {
        let local_path_current = dir.as_ref().unwrap().parent().unwrap().file_name().unwrap();

        // Get all chilled directories in lms [css, vars, svelte, ..etc]
        if let Ok(ref path) = dir {
            if !path.is_dir() {
                continue;
            }

            if local_path_current.eq("grading") {
                continue;
            }

            let node_id = path.file_name().unwrap().to_str().unwrap().to_string();
            let present_node_id = correct_nodes.get(&node_id);

            if present_node_id.is_none() {
                continue;
            }

            match present_node_id.unwrap().as_str().unwrap().to_string() {
                correct_path if !correct_path.eq(local_path_current.to_str().unwrap()) => {
                    let new_name: Vec<_> = correct_path.split("/").collect();

                    let local_path = lms_dir.join(local_path_current).join(&node_id);
                    let valid_path = lms_dir.join(new_name[0]).join(&node_id);

                    if !Path::exists(&valid_path) {
                        misplaced.insert(local_path, valid_path);
                    }
                }
                _ => (),
            }
        }
    }
    misplaced
}
