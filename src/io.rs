use crate::files;

use reqwest::{
    blocking::{Client, Response},
    StatusCode,
};
use serde_json::Value;
use std::{
    os::unix::fs::PermissionsExt,
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
    let exe_name = "lms";

    println!("Current version: {}", env!("CARGO_PKG_VERSION"));

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

    if let Err(err) = fs::remove_file(&lms_loc.join(exe_name)) {
        eprintln!("A error occurred: {}", err);
        exit(1)
    }

    // TODO: Check if macos is arm or intel
    let plat = match env::consts::OS { 
        "linux" => "linux_64",
        "macos" => "mac_arm64",
        _ => {
            eprintln!("Your platform is not supported");
            exit(1)
        }
    };
    
    // TODO: Use reqwest instant of wget 
    execute_command("wget", vec!["-q", "-O", lms_loc.join("lms").to_str().unwrap(), format!("https://github.com/gertjan84/lms-rust-cli/releases/latest/download/lms_{}", plat).as_str()]);
    
    fs::set_permissions(lms_loc.join("lms"), fs::Permissions::from_mode(0o755)).expect("Faild to set permissions");
    println!("Installed");
}
