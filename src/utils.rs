use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::process::{Command, Stdio, exit};
use reqwest::{
    blocking::{Response, Client},
    StatusCode
};



pub fn request(method: &str, path: String, token: &String, data: Option<Vec<u8>>) -> Option<Response>  {

    if Path::exists(&get_lms_dir().join(".latest_version")) {
    }

    let url = if path.contains("?") {
        format!("{}{}&v={}", crate::BASE_URL.to_string(), path, "999")
    } else {
        format!("{}{}?v={}", crate::BASE_URL.to_string(), path, "999")
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
                    exit(1);
                }
                _ => {
                    eprintln!("Server status not handled: {:?}", res.status());
                    exit(1);
                }

            }

        }
        Err(_) => {
            // Request faild because the client is offline
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
        .expect("Faild to start tar process");

    match tar_process.stdin.take() {
        Some(mut stdin) => {

            match res {
                Some(mut unwrap_res) => {
                    let mut res_body = vec![];
                    let _ = unwrap_res.copy_to(&mut res_body);
                    let _ = stdin.write(&res_body);

                }
                None => {
                    eprintln!("Warning: Got no response form server");
                    exit(1)
                }
            }
        },
        None => {
            eprintln!("Faild to get stdin ");
            exit(1)
        }
    }
}

pub fn is_folder_empty(path: &PathBuf) -> std::io::Result<bool> {
    let dir_entris = fs::read_dir(path)?;

    for _ in dir_entris {
        return Ok(false)
    }

    Ok(true)
}

pub fn prompt_yes_no(message: String) -> bool {
    loop {
        println!("{} [Y, n]: ", message);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Faild to get input");

        let trim_input = input.trim().to_lowercase();

        // TODO: refactor to match
        if trim_input.eq("y") ||  trim_input.eq("") {
            return true
        } else if trim_input.eq("n") {
            return false
        } else {
            println!("{}: is not valid", trim_input);        
        }
    }
}


pub fn get_lms_dir() -> PathBuf {
    let mut lms_dir = PathBuf::new();
    lms_dir.push(env::var("HOME").unwrap());
    lms_dir.push("lms");

    lms_dir
}
