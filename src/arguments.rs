use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use std::process::{Command, exit};
use rand::{Rng, distributions::Alphanumeric};
use gethostname::gethostname;
use url::form_urlencoded;
use webbrowser;
use crate::{settings::Settings, utils};

const AUTH_TOKEN_LENGHT: u8 = 69;

pub enum Commands {
    Open,
    Grade,
    Upload,
    Download,
    Template,
    Login
}

impl Commands {
    pub fn get_command(command: String) -> Self {
        match command.to_lowercase().as_str() {
            "open" => Commands::Open,
            "grade" => Commands::Grade,
            "upload" => Commands::Upload,
            "download" => Commands::Download,
            "template" => Commands::Template,
            "login" => Commands::Login,
            _ => {
                eprintln!("Invalid command");
                std::process::exit(1);
            }
        }
    }

    pub fn execute(&self) {
        let settings = Settings::new();
        match self {
            Commands::Open => open_logic(settings),
            Commands::Grade => grade_logic(settings),
            Commands::Upload => upload_logic(settings),
            Commands::Download => download_logic(settings),
            Commands::Template => template_logic(settings),
            Commands::Login => login_logic(settings)
        }
    }
}

fn open_logic(settings: Settings) -> () {
    let out_dir = get_work_location(settings.config.get("auth", "token").unwrap_or("".to_string()));
    if let Err(err) = env::set_current_dir(out_dir) {
        eprintln!("{}", err)
    } else {
       Command::new("nvim").arg(".").status().expect("cant run editor");
    }
}

fn grade_logic(settings: Settings) {
    todo!()
}

fn login_logic(mut settings: Settings) {
    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(AUTH_TOKEN_LENGHT.into())
        .map(char::from)
        .collect();

    settings.set("auth".to_string(), "token".to_string(), token.clone());
    let encoded_host = form_urlencoded::byte_serialize(gethostname().as_encoded_bytes()).collect::<String>();
    let url = format!("{}/api/authorize?host={}&token={}", crate::BASE_URL.to_string(), encoded_host, &token);
    println!("Go to this URL to authorize lms: {}", url);
    webbrowser::open(url.as_str());
}

fn upload_logic(settings: Settings) {
    let platform = std::env::consts::OS;

    let cmd = if cfg!(platform = "macos") {
        "gtar"
    } else {
        "tar"
    };
    
    let mut tar = Command::new(cmd);
    tar.arg("czC")
        .arg(get_work_location(settings.config.get("auth", "token").unwrap()))
        .arg("--exclude-backups")
        .arg("--exclude-ignore=.gitignore")
        .arg("--exclude-ignore=.lmsignore")
        .arg(".");

    let output = match tar.output() {
        Ok(output) => output,
        Err(_) => {
            eprintln!("Command not found: {}", cmd);
            if cfg!(platform = "macos") {
                println!("Please install gnu-tar (using brew for instanse")
            }
            exit(1)
        }
    };
}


fn download_logic(settings: Settings) {
    todo!()
}

fn template_logic(settings: Settings) {
    todo!()
}

fn get_work_location(token: String) -> PathBuf {
    let mut lms_dir = PathBuf::new();
    lms_dir.push(env::var("HOME").unwrap());
    lms_dir.push("lms");

    let mut cache = lms_dir.clone();
    cache.push(".cache");

    let online_attempt = utils::request("/api/attempts/current".to_string(), token, "".to_string());

    if online_attempt.is_none() {
        if Path::exists(&cache) {
            let cache_location = match fs::read_to_string(&cache) {
                Ok(cache_content) => cache_content.to_string(),
                Err(_) => {
                    eprintln!("No cached assignment");
                    exit(1)
                }
            };

            cache.push(cache_location);
            return cache
        } 

        eprintln!("No cache file");
        exit(1)
    }

    let assignment_path = &online_attempt.unwrap();
    let json_value = &assignment_path.get("path").unwrap().as_str().unwrap();
    lms_dir.push(json_value);

    return lms_dir;
}
