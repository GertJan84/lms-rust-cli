use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use std::process::{Command, exit, Stdio};
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
                exit(1);
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
    match env::set_current_dir(out_dir) {
        Ok(_) =>{
            settings.editors.iter().for_each(|editor| {
                if Command::new("which").arg(editor).stdout(Stdio::null()).status().expect("Can't find which").success() {
                    Command::new(format!("{}", editor))
                        .arg(".")
                        .status()
                        .expect("Failed to execute editor");
                    exit(0)
                }
            })
        },
        Err(err) => eprintln!("{}", err)
    }
}

fn grade_logic(settings: Settings) {
    todo!("Implelent grade logic")
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
    let _ = webbrowser::open(url.as_str());
}

fn upload_logic(settings: Settings) {

    // TODO: Use tar crate
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

    // TODO: Look over this again (output instand of status)?
    match tar.output() {
        Ok(output) => output,
        Err(_) => {
            eprintln!("Command not found: {}", cmd);
            if cfg!(platform = "macos") {
                println!("Please install gnu-tar (using brew for instanse")
            }
            exit(1)
        }
    };

    todo!("Send data to server")
}


fn download_logic(settings: Settings) {
    let token = settings.config.get("auth", "token").unwrap_or("".to_string());
    let response = utils::request("/api/attempts/@".to_string(), &token, "".to_string());

    // TODO: Doesn't get current attempt
    let attempt = match response {
        Some(data) => utils::response_to_json(data),

        None => {
            eprintln!("no attempt found");
            exit(1)
        }
    };

    // TODO: debug get value of function 
    utils::download_tgz("/api/attempts/fasnjkl@vars:1/submission".to_string(), &token, PathBuf::new())
    // let out_dir = get_work_location(token);

    // if Path::exists(&out_dir) {
    //     eprintln!("output directory {} already exists", out_dir.to_str().unwrap());
    //     exit(1)
    //}

    //fs::create_dir(out_dir);
}

fn template_logic(settings: Settings) {
    todo!("Implement template logic")
}

fn get_work_location(token: String) -> PathBuf {
    let mut lms_dir = PathBuf::new();
    lms_dir.push(env::var("HOME").unwrap());
    lms_dir.push("lms");

    let mut cache = lms_dir.clone();
    cache.push(".cache");


    // TODO: Make let if
    let res = utils::request("/api/attempts/current".to_string(), &token, "".to_string());

    if res.is_none() {
        if Path::exists(&cache) {
            let cache_location = match fs::read_to_string(&cache) {
                Ok(cache_content) => cache_content.to_string(),
                Err(_) => {
                    eprintln!("No cached assignment");
                    exit(1)
                }
            };

            lms_dir.push(cache_location);
            return lms_dir
        } 

        eprintln!("No cache file");
        exit(1)
    }

    let online_attempt = utils::response_to_json(res.unwrap());
    let assignment_path = &online_attempt;
    let json_value = &assignment_path.get("path").unwrap().as_str().unwrap();

    match fs::write(&cache, json_value) {
        Ok(_) => {},
        Err(err) => eprintln!("Can't write to cache because: {}", err)
    }

    lms_dir.push(json_value);

    return lms_dir;
}
