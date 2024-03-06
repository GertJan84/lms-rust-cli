use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Stdio, exit};
use reqwest::{
    blocking::{Response, Client},
    StatusCode
};



pub fn request(path: String, token: &String, data: String) -> Option<Response>  {
    let url = if path.contains("?") {
        format!("{}{}&v={}", crate::BASE_URL.to_string(), path, crate::CLI_VERSION)
    } else {
        format!("{}{}?v={}", crate::BASE_URL.to_string(), path, crate::CLI_VERSION)
    };

    let client = Client::new();

    let res = client
        .get(url)
        .header("authorization", token)
        .send();

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
                    eprintln!("You dont have the right to access this");
                    exit(1)
                }


                StatusCode::IM_A_TEAPOT => {
                    // TODO: Update client
                    todo!("Client is outdated: utils teapot");
                }
                _ => {
                    eprintln!("Server status not handled: {:?}", res.status());
                    exit(1);
                }

            }

        }
        Err(err) => {
            return None
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

pub fn download_tgz(path: String, token: &String, out_dir: PathBuf) -> () {
    let res = request(path, token, "".to_string());

    match res {
        Some(res) => {
            println!("{:?}", res.text());
        }
        None => exit(1)
    }

    //let mut tar_process = Command::new("tar")
    //    .arg("xzC")
    //    .arg(out_dir)
    //    .stdin(Stdio::piped())
    //    .spawn()
    //    .expect("Faild to start tar process");

    //let stdin = tar_process.stdin.take().ok_or("Faild to open tar stdin");

}
