use std::{
    path::Path,
};
use std::process::exit;
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

    if files::is_folder_empty(&current_attempt.get_path_buf()) {
        if !prompt::yes_no("This folder is currently empty are you sure you want to upload?") {
            return eprintln!("Cancelled upload");
        }
    }

    let data = io::compress_folder(&current_attempt.get_path_buf()); 
    
    
    if data.is_none() {
        error_exit!("Can't comporess folder: {}", &current_attempt.get_path_buf().to_str().unwrap())
    } 
    

    let url = format!(
        "/api/attempts/{}/submission",
        current_attempt.id.to_string()
    );

    match io::request("POST", url, &settings.get_token(), data) {
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
