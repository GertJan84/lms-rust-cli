use reqwest::{
    blocking::{Client, Response},
    StatusCode,
};
use serde_json::Value;
use std::{
    env,
    fs::{self, File},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{exit, Command, Stdio},
};

use crate::error_exit;

const SUPPORTED_ARCHITECTURES: [&str; 2] = ["x86_64", "aarch64"];

// TODO: Implement tests for request
pub fn request(
    method: &str,
    path: String,
    token: &String,
    data: Option<Vec<u8>>,
) -> Option<Response> {
    let url = format!(
        "{}{path}{}v=999",
        crate::BASE_URL.to_string(),
        if path.contains("?") { "&" } else { "?" }
    );

    let client = Client::new();

    let res = match method {
        "GET" => client.get(url).header("authorization", token).send(),
        "POST" => client
            .post(url)
            .header("authorization", token)
            .body(data.clone().unwrap())
            .send(),
        _ => error_exit!("Invalid method: {}", method),
    };

    match res {
        Ok(res) => match res.status() {
            StatusCode::OK => Some(res),
            StatusCode::UNAUTHORIZED => error_exit!("You are not logged in"),
            StatusCode::FORBIDDEN => error_exit!("You don't have the right to access this"),
            StatusCode::IM_A_TEAPOT => {
                println!("Updating client ...");
                handle_upgrade();
                println!("done");
                exit(0);
            }
            _ => error_exit!("Server status not handled {:?}", res.status()),
        },
        Err(_) => {
            println!("Request failed because the client is offline");
            None
        }
    }
}

// TODO: Implement tests for response_to_json
pub fn response_to_json(res: Response) -> Value {
    let text = res.text().unwrap();
    match serde_json::from_str(&text) {
        Ok(data) => data,
        Err(err) => error_exit!("JSON parsing error: \n{}", err),
    }
}

pub fn is_installed(application: &str) -> bool {
    return execute_command("which", vec![application]);
}

// TODO: Implement tests for execute_command
pub fn execute_command(application: &str, args: Vec<&str>) -> bool {
    return Command::new(application)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("error on executing command")
        .success();
}

// TODO: Implement tests for download_tgz
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
            None => error_exit!("Warning: Got no response from server"),
        },
        None => error_exit!("Failed to get stdin"),
    }

    drop(tar_process)
}

// TODO: Implement tests for handle_upgrade
pub fn handle_upgrade() {
    let exe_name = "lms";

    println!("Current version: {}", env!("CARGO_PKG_VERSION"));

    let mut lms_loc = PathBuf::new();
    lms_loc.push(env::var("HOME").unwrap());
    lms_loc.push(".local");
    lms_loc.push("bin");

    if !Path::exists(&lms_loc) {
        if let Err(err) = fs::create_dir_all(&lms_loc) {
            error_exit!("A error occurred with creating: {}", err);
        }
    }

    let architecture = std::env::consts::ARCH;

    if !SUPPORTED_ARCHITECTURES.contains(&architecture) {
        error_exit!("{} processor is not supported", &architecture);
    }

    // TODO: Check if macos is arm or intel
    let plat = match env::consts::OS {
        "linux" => "linux_x64",
        "macos" => "mac_arm64",
        platform => error_exit!("Your platform is not supported - {}", platform),
    };

    let mut response = reqwest::blocking::get(format!(
        "https://github.com/gertjan84/lms-rust-cli/releases/latest/download/lms_{}",
        plat
    ))
    .expect("request failed");

    if !response.status().is_success() {
        error_exit!("Failed to download lms");
    }

    if Path::exists(&lms_loc.join(exe_name)) {
        println!("Removing old version");
        if let Err(err) = fs::remove_file(&lms_loc.join(exe_name)) {
            error_exit!("A error occurred with removing: {}", err);
        }
    }

    let mut file = File::create(lms_loc.join("lms")).expect("failed to create file");
    response
        .copy_to(&mut file)
        .expect("failed to write to file");

    fs::set_permissions(lms_loc.join("lms"), fs::Permissions::from_mode(0o755))
        .expect("Failed to set permissions");

    println!("Installed");
}
