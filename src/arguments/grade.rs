use glob::glob;
use std::{
    env, fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use crate::{files, io, settings::Settings, ustring};

use super::ide::open_ide;

pub fn grade_logic(settings: &Settings, arg: String) {
    let token = settings.get_token();
    let url_arg = format!("/api/attempts/{}", arg.replace("~", ":"));
    let response = io::request("GET", url_arg, &token, None);

    let attempts = match response {
        Some(data) => io::response_to_json(data),
        None => {
            return eprintln!("No attempt found");
        }
    };

    let attempt = &attempts[0];

    let out_dir = files::get_lms_dir().join("grading").join(
        attempt
            .get("spec")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
            .replace(":", "~"),
    );

    if Path::exists(&out_dir) {
        if files::is_folder_empty(&out_dir) {
            match fs::remove_dir_all(&out_dir) {
                Ok(_) => {}
                Err(err) => eprintln!("Cant remove directory because: {}", err),
            }
        }
    }

    if Path::exists(&out_dir) {
        eprintln!(
            "Submission already exists in {}",
            ustring!(out_dir.to_str())
        )
    } else {
        let _ = fs::create_dir_all(&out_dir);
        let url = format!(
            "/api/attempts/{}/submission",
            ustring!(attempt.get("spec").unwrap().as_str())
        );
        io::download_tgz(url, &token, &out_dir);
        println!("Downloaded to {}", ustring!(out_dir.to_str()));
    }

    for name in vec!["_node", "_solution", "_template"] {
        let _ = fs::remove_dir_all(&out_dir.join(name));

        let mut curriculum_dir = PathBuf::new();
        curriculum_dir.push(env::var("HOME").unwrap());
        curriculum_dir.push(settings.get_string(
            "grade",
            "curriculum_directory",
            "curriculum".to_string(),
        ));

        let mut glob_path = PathBuf::new();
        glob_path.push(&curriculum_dir);
        glob_path.push(ustring!(&attempt.get("period")));
        glob_path.push(ustring!(&attempt.get("module_id")));
        glob_path.push(format!("[0-9][0-9]-{}", ustring!(&attempt.get("node_id"))));

        let glob_str = glob_path.to_str().expect("Invalid UTF-8 in path");
        if let Ok(mut paths) = glob(&glob_str) {
            match paths.next() {
                Some(found_node_id) => {
                    let node_id = found_node_id.unwrap();
                    let _ = symlink(
                        &node_id,
                        out_dir.join(format!("_{}", ustring!(node_id.to_str()))),
                    );

                    for what in vec!["solution", "template"] {
                        let what_dir = out_dir.join(format!(
                            "{}{}",
                            what,
                            ustring!(attempt.get("variant_id").unwrap().as_str())
                        ));
                        if let Ok(metadata) = fs::metadata(&what_dir) {
                            let _ = metadata
                                .is_dir()
                                .then(|| symlink(&what, out_dir.join(format!("_{}", what))))
                                .expect("Failed to create symlink");
                        };
                    }
                }
                None => {}
            }
        }
    }

    open_ide(&out_dir, &settings.editors)
}
