use crate::main;
use reqwest::{
    blocking::Client,
    StatusCode
};



pub fn request(path: String, token: String, data: String) -> () {
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

                }

                StatusCode::IM_A_TEAPOT => {
                    // TODO: Update client
                    todo!();
                }
                _ => eprintln!("Server status not handled: {:?}", res.status()) 

            }

        }
        Err(err) => println!("request error: {:?}", err)    
    }
}
