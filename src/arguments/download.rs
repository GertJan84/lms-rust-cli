use glob::glob;
use std::{collections::HashSet, fs, path::Path, thread::sleep, time::Duration};

use crate::{attempt::Attempt, files, io, settings::Settings};

use super::DOWNLOAD_EXCLUDE;

pub fn download_logic(settings: &Settings, arg: String) {
    let token = settings.get_token();

    if !arg.eq("all") {
        let _ = download_attempt(&arg, &token);
    }

    let response = io::request("GET", "/api/node-paths".to_string(), &token, None);
    let attempts = match response {
        Some(data) => io::response_to_json(data),
        None => {
            return eprintln!("No attempt found");
        }
    };

    let mut local_dirs: HashSet<String> = HashSet::new();
    let target_dir = files::get_lms_dir().join("*/*");

    for path in glob(target_dir.to_str().unwrap()).expect("Failed to read lms dir") {
        match path {
            Ok(path) => {
                local_dirs.insert(
                    path.as_path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                );
            }
            Err(_) => {}
        }
    }

    attempts.as_array().unwrap()[0]
        .as_object()
        .unwrap()
        .iter()
        .for_each(|(assignment, _)| {
            let mut ignore = false;

            for exclude in DOWNLOAD_EXCLUDE {
                if assignment.contains(exclude) {
                    ignore = true;
                    break;
                }
            }

            if !local_dirs.contains(assignment) && !ignore {
                download_attempt(&assignment.to_string(), &token);
                sleep(Duration::from_millis(500));
            }
        })
}

pub fn download_attempt(assignment: &String, token: &String) -> bool {
    let url_arg = format!("/api/attempts/@{}", assignment.replace("~", ":"));
    let response = io::request("GET", url_arg, token, None);

    let attempts = match response {
        Some(data) => io::response_to_json(data),
        None => {
            eprintln!("No attempt found: {}", assignment);
            return false;
        }
    };

    let attempt = &attempts[0];

    match attempt.as_object() {
        Some(select_attempt) => {
            let mut out_dir = files::get_lms_dir();

            match select_attempt.get("path") {
                Some(att) => {
                    out_dir.push(att.as_str().unwrap());

                    if Path::exists(&out_dir) {
                        eprintln!(
                            "Output directory {} already exists",
                            out_dir.to_str().unwrap()
                        );
                        return false;
                    }

                    let select_attempts = select_attempt.get("spec").unwrap().clone();

                    let _ = fs::create_dir_all(&out_dir);

                    let url = format!(
                        "/api/attempts/{}/submission",
                        select_attempts.as_str().unwrap()
                    );
                    io::download_tgz(url, &token, &out_dir);
                    println!(
                        "Downloaded: {} at: {}",
                        assignment,
                        &out_dir.to_str().unwrap()
                    );
                }
                None => return false,
            }
        }

        None => {
            eprintln!("Cant find attempt: {}", assignment);
            return false;
        }
    }
    return true;
}

pub fn download_template(token: &String, attempt: &Attempt) -> bool {
    if !Path::exists(&attempt.get_path_buf()) {
        let _ = fs::create_dir_all(&attempt.get_path_buf());
        println!("Created {}", &attempt.get_path_buf().to_str().unwrap());
    } else {
        if !files::is_folder_empty(&attempt.get_path_buf()) {
            return false;
        }
    }

    if attempt.offline {
        println!("No connection to server");
        return false;
    }

    let url = format!("/api/attempts/{}/template", &attempt.id);
    io::download_tgz(url, &token, &attempt.get_path_buf());
    true
}
