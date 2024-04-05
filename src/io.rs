use serde_json::Value;
use std::path::PathBuf;
use std::io::{self, Write};
use std::process::{Command, Stdio, exit};
use reqwest::{
    blocking::{Response, Client},
    StatusCode
};

use crate::CLI_VERSION;



pub fn request(method: &str, path: String, token: &String, data: Option<Vec<u8>>) -> Option<Response>  {

    let url = if path.contains("?") {
        format!("{}{}&v={}", crate::BASE_URL.to_string(), path, CLI_VERSION)
    } else {
        format!("{}{}?v={}", crate::BASE_URL.to_string(), path, CLI_VERSION)
    };

    let client = Client::new();

    let res = match method {
        "GET" => client.get(url).header("authorization", token).send(),
        "POST" => client.post(url).header("authorization", token).body(data.unwrap()).send(),
        _=> {
            eprintln!("Invalid method: {}", method);
            exit(1)
        }
    };


    match res {
        Ok(res) => {
            match res.status() {
                StatusCode::OK => {
                    Some(res)
                }

                StatusCode::UNAUTHORIZED => {
                    eprintln!("You are not logged in");
                    exit(1)
                }

                StatusCode::FORBIDDEN => {
                    eprintln!("You don't have the right to access this");
                    exit(1)
                }


                StatusCode::IM_A_TEAPOT => {
                    // TODO: Update client (optional)
                    println!("Client needs to be updated");

                    // let output = Command::new("wget")
                    // .args(&["-qO-", "https://gitlab.com/gj-535479/lms-rust-cli/-/raw/nightly/install"])
                    // .output();
                                    
                    // let python_script = String::from_utf8(output.unwrap().stdout);
                    
                    // let _ = Command::new("python")
                    //     .arg("-c")
                    //     .arg(&python_script.unwrap())
                    //     .output();
                    
                    exit(1)
                }
                _ => {
                    eprintln!("Server status not handled: {:?}", res.status());
                    exit(1);
                }

            }

        }
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

pub fn download_tgz(path: String, token: &String, out_dir: &PathBuf) -> () {
    let res = request("GET", path, token, None);

    let cmd = if cfg!(target_os = "macos") {
        "gtar"
    } else {
        "tar"
    };

    if res.is_none() {
        return
    }

    let mut tar_process = Command::new(cmd)
        .arg("xzC")
        .arg(out_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .expect("Failed to start tar process");

    match tar_process.stdin.take() {
        Some(mut stdin) => {
            match res {
                Some(mut unwrap_res) => {
                    let mut res_body = vec![];
                    let _ = unwrap_res.copy_to(&mut res_body);
                    let _ = stdin.write(&res_body);

                }
                None => {
                    eprintln!("Warning: Got no response from server");
                    exit(1)
                }
            }
        },
        None => {
            eprintln!("Failed to get stdin");
            exit(1)
        }
    }

    drop(tar_process)
}
