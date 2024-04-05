use std::{env, fs, path::PathBuf};

pub fn get_lms_dir() -> PathBuf {
    let mut lms_dir = PathBuf::new();
    lms_dir.push(env::var("HOME").unwrap());
    lms_dir.push("lms");

    lms_dir
}

pub fn is_folder_empty(path: &PathBuf) -> std::io::Result<bool> {
    let dir_entries = fs::read_dir(path)?;

    for _ in dir_entries {
        return Ok(false)
    }

    Ok(true)
}