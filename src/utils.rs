use crate::main;
use std::collections::HashMap;
use std::process::exit;
use reqwest::{
    blocking::Client,
    StatusCode
};



pub fn request(path: String, token: String, data: String) -> Option<HashMap<String, serde_json::Value>>  {
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
                    match res.json::<HashMap<String, serde_json::Value>>() {
                        Ok(data) => Some(data),
                        Err(_) => {
                            return None
                        }
                   }
                }

                StatusCode::UNAUTHORIZED => {
                    eprintln!("You are not logged in");
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
            println!("request error: {:?}", err);
            return None
        }
    }
}
