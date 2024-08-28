use std::{env, fs, path::Path, process::{Command, Stdio, exit}};
use std::path::PathBuf;
use crate::{attempt::Attempt, error_exit, files, io, prompt, settings::Settings, stru, ustring};

use super::{
    download::download_template,
    ide::open_ide,
    setups::{get_todo, verify_logic},
};

pub fn open_logic(settings: &Settings) -> () {
    let current_attempt = Attempt::get_current_attempt(&settings);

    if !download_template(&settings.get_token(), &current_attempt) {
        println!(
            "Already exists in {}",
            ustring!(&current_attempt.get_path_buf().to_str())
        );
    }

    if settings.get_bool("setup", "move_node_directories", true) {
        verify_logic()
    }

    open_ide(&current_attempt.get_path_buf(), &settings.editors)
}

pub fn upload_logic(settings: &Settings) {
    let current_attempt = Attempt::get_current_attempt(settings);

    if !Path::exists(&current_attempt.get_path_buf()) {
        eprintln!(
            "There is no folder: {}",
            stru!(current_attempt.get_path_buf())
        );
        return eprintln!("Try `lms template` first");
    }

    if settings.get_bool("setup", "check_todo", true) {
        if let Some(file_todo) = get_todo(&current_attempt.get_path_buf()) {
            println!("You still have some TODO's in your code: ");
            for (file, todos) in file_todo {
                println!("\n{}: has some TODO's:", file);

                for (idx, line) in todos {
                    println!("  {} -> {}", idx, line)
                }
            }

            if prompt::yes_no("\nYou still have some TODO's in your code do you want to fix them") {
                return println!("Upload cancelled");
            }
        }
    }

    let cmd = if cfg!(target_os = "macos") {
        "gtar"
    } else {
        "tar"
    };

    if files::is_folder_empty(&current_attempt.get_path_buf()) {
        if !prompt::yes_no("This folder is currently empty are you sure you want to upload?") {
            return eprintln!("Cancelled upload");
        }
    }

    let mut tar = Command::new(cmd);
    tar.arg("czC")
        .arg(ustring!(current_attempt.get_path_buf().to_str()))
        .arg("--exclude-backups")
        .arg("--exclude-ignore=.gitignore")
        .arg("--exclude-ignore=.lmsignore")
        .arg(".")
        .stdin(Stdio::null())
        .stdout(Stdio::piped());

    let data = match tar.output() {
        Ok(output) => output,
        Err(_) => {
            if cfg!(platform = "macos") {
                return println!("Please install gnu-tar (using brew for instance)");
            }
            return eprintln!("Command not found: {}", cmd);
        }
    };

    let url = format!(
        "/api/attempts/{}/submission",
        current_attempt.id.to_string()
    );

    match io::request("POST", url, &settings.get_token(), Some(data.stdout)) {
        Some(res) => {
            let json_res: serde_json::Value = io::response_to_json(res);

            match json_res.get("transferred") {
                Some(transferred) => {
                    if let Some(upload_bytes) = transferred.as_u64() {
                        let upload_kb = upload_bytes / 1024;
                        println!("Uploaded complete: {}kb transferred", upload_kb);

                        if settings.get_bool("setup", "upload_open_browser", true) {
                            let _ = webbrowser::open(&current_attempt.get_url());
                        } else {
                            println!("Please remember that you still need to submit in the web interface");
                        }
                    }
                }
                None => return eprintln!("Error getting transferred value"),
            }
        }
        None => return eprintln!("Failed to upload attempt"),
    }
}

pub fn template_logic(settings: &Settings) {
    let current_attempt = Attempt::get_current_attempt(settings);

    if !download_template(&settings.get_token(), &current_attempt) {
        let error_message = format!(
            "Output directory {} already exists",
            ustring!(current_attempt.get_path_buf().to_str())
        );

        return eprintln!("{}", error_message);
    }
}

pub fn install_logic() {
    let mut lms_loc = PathBuf::new();
    lms_loc.push(env::var("HOME").unwrap());
    lms_loc.push(".local");
    lms_loc.push("bin");

    let download_location = env::current_exe().unwrap();
    let lms_loc_str = lms_loc.to_str().unwrap();

    if download_location != lms_loc {
        println!("Move lms to recommended folder {}", lms_loc_str);
        if let Err(err) = fs::rename(download_location.to_str().unwrap(), lms_loc.join("lms").to_str().unwrap()) {
            error_exit!("{}", err)
        }
    }

    if let  Some(path) = env::var_os("PATH") {
       if path.to_string_lossy().contains(&lms_loc_str) {
           return println!("Install complete")
       }

        println!("This system doesn't have {} in its PATH.", lms_loc_str);
        println!("  Add this before running lms");

        if let Some(shell) = env::var_os("SHELL") {
            println!("Execute this command to add lms to your path");
            match shell.to_string_lossy().split('/').last() {
                Some("zsh") => {
                    println!("echo path += '{}' >> ~/.zshrc", lms_loc_str);
                    println!("echo EXPORT PATH >> ~/.zshrc");
                }
                Some("bash") => {
                    println!(r"echo PATH={}:\$PATH >> ~/.bashrc", lms_loc_str);
                }
                _ => {
                    eprintln!("You need to add {} to your shell", lms_loc_str);
                }
            }
        } else {
            eprintln!("SHELL environment variable is not set.");
        }
    } else {
        eprintln!("PATH environment variable is not set.");
    }
}
