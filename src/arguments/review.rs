use crate::{arguments::setups, attempt::Attempt, io, settings::Settings, ustr_ustring, ustring};
use colored::*;
use reqwest::{
    blocking::Client,
    header::{AUTHORIZATION, CONTENT_TYPE},
    StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::exit;

#[derive(Debug, Serialize, Deserialize)]
struct AIPayload {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    type_reference: String,
    line: u32,
    column: u32,
    message: String,
    suggestion: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileErrors {
    filename: String,
    content: Vec<Content>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    files: Vec<FileErrors>,
}

pub fn review(settings: &Settings) {
    let ai_endpoint = settings.get_string("ai", "endpoint", "".to_string());
    let ai_key = settings.get_string("ai", "key", "".to_string());

    if ai_key.is_empty() || ai_endpoint.is_empty() {
        return eprintln!("You haven't defined your key or endpoint");
    }

    let attempt = Attempt::get_current_attempt(&settings);
    let mut files_parsed: HashMap<String, String> = HashMap::new();

    let found_files = match setups::get_attempt_files_content(&attempt.get_path_buf()) {
        Some(data) => data,
        None => return eprintln!("Can't find files"),
    };

    found_files.iter().for_each(|(path, content)| {
        let mut file_content: String = String::new();

        content
            .iter()
            .for_each(|line| file_content.push_str(format!("{}\n", line).as_str()));

        files_parsed.insert(ustr_ustring!(path.file_name()), file_content);
    });

    let binding = serde_json::to_string(&files_parsed).unwrap();
    let files = binding.as_str();

    let req = AIPayload {
        model: "gpt-4-turbo".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a code reviewer and are reviewing code that a student has submitted. Your goal is to help the student by reviewing there code on spelling errors, bugs, vulnerabilityes and recommend patterns to imporove performanse and speed".to_string(),
            },

            // TODO: Create macro to import json content from different file
            Message {
                role: "system".to_string(),
                content: r#"Pretent you are a api and respond with the following json compressed format example: {"files": [{"filename": "example.py", "content": [{"type_reference":"error","line":5,"column":10,"message":"IndentationError: expected an indented block","suggestion":"Indent the code block starting at line 5."},{"type_reference":"good_job","line":20,"column":5,"message":"Well-documented code","suggestion":"Keep up the good work!"}]},{"filename": "example.js", "content": [{"type_reference":"warning","line":12,"column":3,"message":"Using var","suggestion":"Use 'let' or const instead"},{"type_reference":"spelling","line":3,"column":2,"message":"Variable name 'colur' should be spelled 'color'","suggestion":"Correct the spelling of 'colur' to 'color'."}]},{"filename": "example.sql", "content": [{"type_reference":"vulnerability","line":10,"column":8,"message":"Potential SQL injection vulnerability","suggestion":"Consider using parameterized queries or prepared statements to sanitize user input."}]}]}"#.to_string(),
            },

            Message {
                role: "user".to_string(),
                content: files.to_string(),
            },
        ],
    };

    let request_json = serde_json::to_string(&req).unwrap();

    let client = Client::new();

    println!("Reviewing code ...\n");
    let res = client
        .post(&ai_endpoint)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", &ai_key))
        .body(request_json)
        .send();

    if let Err(err) = res {
        eprintln!("Failed to chat with ai: {}", err);
        return;
    }

    let response = match res.as_ref().unwrap().status() {
        StatusCode::OK => res.unwrap(),

        StatusCode::UNAUTHORIZED => {
            eprintln!("Key is invalid");
            exit(1)
        }

        StatusCode::NOT_FOUND => {
            eprintln!("Route of endpoint not found: {}", &ai_endpoint);
            exit(1)
        }

        StatusCode::SERVICE_UNAVAILABLE => {
            eprintln!("AI api is unavalible");
            exit(1)
        }
        _ => {
            eprintln!("Response not handles: {}", res.unwrap().status());
            exit(1)
        }
    };

    let json_data = io::response_to_json(response);
    let res_array = json_data.as_object().unwrap().get("choices");

    let tmp = res_array
        .unwrap()
        .as_array()
        .unwrap()
        .get(0)
        .unwrap()
        .as_object()
        .unwrap()
        .get("message");
    let data = tmp
        .unwrap()
        .as_object()
        .unwrap()
        .get("content")
        .unwrap()
        .as_str()
        .unwrap();

    let parse_response: Result<Response, serde_json::Error> = serde_json::from_str(data);

    match parse_response {
        Ok(data) => pretty_diff_print(found_files, data),
        Err(_) => eprintln!("Can't parse ai response, try again later"),
    }
}

fn pretty_diff_print(files: HashMap<PathBuf, Vec<String>>, ai_response: Response) -> () {
    let mut parse_files: HashMap<String, Vec<String>> = HashMap::new();
    files.iter().for_each(|(file, content)| {
        let file_data = content.clone();

        parse_files.insert(ustr_ustring!(file.file_name()), file_data);
    });

    for rec in ai_response.files {
        let file_content = match parse_files.get(rec.filename.as_str()) {
            Some(data) => data,
            None => continue,
        };

        println!("-----------------------------");
        for recommendation in rec.content {
            match recommendation.type_reference.as_str() {
                "error" => {
                    println!("{}", "Error".red())
                }

                "warning" => {
                    println!("{}", "Warning".yellow())
                }

                "spelling" => {
                    println!("{}", "Spelling".blue())
                }

                "good_job" => {
                    println!("{}", "Good Job".green())
                }

                "vulnerability" => {
                    println!("{}", "Vulnerability".purple())
                }

                _ => {
                    println!("Unknown: {}", recommendation.type_reference.as_str())
                }
            }
            println!("\n{}\n", recommendation.message.as_str().underline());

            let line_index = recommendation.line as usize;

            if line_index > 0 && line_index < file_content.len() {
                let start = line_index.saturating_sub(1);
                let end = (line_index + 1).min(file_content.len());

                let slice = &file_content[start..end];

                for line in slice {
                    println!("{}", line)
                }
            } else {
                eprintln!("Can't print file content reference on: {}", line_index);
            }

            println!("\n{}\n", recommendation.suggestion.as_str().underline());

            println!("-----------------------------");
        }
    }
}
