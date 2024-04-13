use crate::files;
use reqwest::{
    blocking::{Client, Response},
    StatusCode,
};
use serde_json::Value;
use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{exit, Command, Stdio},
};

// use crate::CLI_VERSION;

pub fn request(
    method: &str,
    path: String,
    token: &String,
    data: Option<Vec<u8>>,
) -> Option<Response> {
    let url = if path.contains("?") {
        format!("{}{}&v={}", crate::BASE_URL.to_string(), path, "999")
    } else {
        format!("{}{}?v={}", crate::BASE_URL.to_string(), path, "999")
    };

    let client = Client::new();

    let res = match method {
        "GET" => client.get(url).header("authorization", token).send(),
        "POST" => client
            .post(url)
            .header("authorization", token)
            .body(data.clone().unwrap())
            .send(),
        _ => {
            eprintln!("Invalid method: {}", method);
            exit(1)
        }
    };

    match res {
        Ok(res) => match res.status() {
            StatusCode::OK => Some(res),

            StatusCode::UNAUTHORIZED => {
                eprintln!("You are not logged in");
                exit(1)
            }

            StatusCode::FORBIDDEN => {
                eprintln!("You don't have the right to access this");
                exit(1)
            }

            StatusCode::IM_A_TEAPOT => {
                println!("Updating client ...");
                handle_upgrade();
                println!("done");
                exit(0)
            }
            _ => {
                eprintln!("Server status not handled: {:?}", res.status());
                exit(1);
            }
        },
        Err(_) => {
            println!("Request failed because the client is offline");
            None
        }
    }
}

pub fn response_to_json(res: Response) -> Value {
    let text = res.text().unwrap();
    match serde_json::from_str(&text) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("JSON parsing error: \n{}", err);
            exit(1)
        }
    }
}

pub fn is_installed(application: &str) -> bool {
    return execute_command("which", vec![application]);
}

pub fn execute_command(application: &str, args: Vec<&str>) -> bool {
    return Command::new(application)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("error on executing command")
        .success();
}

pub fn download_tgz(path: String, token: &String, out_dir: &PathBuf) -> () {
    let res = request("GET", path, token, None);

    let cmd = if cfg!(target_os = "macos") {
        "gtar"
    } else {
        "tar"
    };

    if res.is_none() {
        return;
    }

    let mut tar_process = Command::new(cmd)
        .arg("xzC")
        .arg(out_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .expect("Failed to start tar process");

    match tar_process.stdin.take() {
        Some(mut stdin) => match res {
            Some(mut unwrap_res) => {
                let mut res_body = vec![];
                let _ = unwrap_res.copy_to(&mut res_body);
                let _ = stdin.write(&res_body);
            }
            None => {
                eprintln!("Warning: Got no response from server");
                exit(1)
            }
        },
        None => {
            eprintln!("Failed to get stdin");
            exit(1)
        }
    }

    drop(tar_process)
}

pub fn handle_upgrade() {
    let repo_url = env!("CARGO_PKG_REPOSITORY");
    let exe_name = "lms";
    let tmp_loc = Path::new("/tmp/lms_rust");

    println!("Current version: {}", env!("CARGO_PKG_VERSION"));

    if !is_installed("git") {
        eprintln!("Git it not installed");
        exit(1)
    }

    if Path::exists(&tmp_loc) {
        if let Err(err) = fs::remove_dir_all(tmp_loc) {
            eprintln!("Can't remove tmp folder: {}", err)
        }
    }

    if let Err(err) = fs::create_dir_all(tmp_loc) {
        eprintln!("A error occurred: {}", err);
        exit(1)
    }

    execute_command("git", vec!["clone", repo_url, tmp_loc.to_str().unwrap()]);

    if files::is_folder_empty(&tmp_loc.to_path_buf()) {
        eprintln!("Can't clone new version");
        exit(1)
    }

    println!("Cloned new version");
    if let Err(err) = env::set_current_dir(tmp_loc) {
        eprintln!("A error occurred: {}", err);
        exit(1)
    }

    execute_command("cargo", vec!["build", "--release", "--quiet"]);

    let mut lms_loc = PathBuf::new();
    lms_loc.push(env::var("HOME").unwrap());
    lms_loc.push(".local");
    lms_loc.push("bin");

    if !Path::exists(&lms_loc) {
        if let Err(err) = fs::create_dir_all(&lms_loc) {
            eprintln!("A error occurred: {}", err);
            exit(1)
        }
    }

    let new_compiled_exec = tmp_loc.join("target").join("release").join(exe_name);

    if !Path::exists(&new_compiled_exec.as_path()) {
        eprintln!("Can't find new compiled lms");
        exit(1);
    }

    println!("Compiled new version");

    if let Err(err) = fs::remove_file(&lms_loc.join(exe_name)) {
        eprintln!("A error occurred: {}", err);
        exit(1)
    }

    match fs::copy(&new_compiled_exec.as_path(), lms_loc.join(exe_name)) {
        Ok(_) => {
            println!("LMS updated to a new version");
        }
        Err(err) => {
            eprintln!("A error occurred: {}", err);
            exit(1)
        }
    }

    if let Err(err) = fs::remove_dir_all(tmp_loc) {
        eprintln!("A error occurred: {}", err);
        exit(1)
    }
}
