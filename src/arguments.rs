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

struct Attempt {
    path: PathBuf,
    spec: String,
    id: String
}

impl Attempt {
    pub fn new(path: PathBuf, spec: String, id: String) -> Self {
        Self {
            path,
            spec,
            id
        }
    }
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

    pub fn execute(&self, arg: String) {
        let settings = Settings::new();
        match self {
            Commands::Open => open_logic(settings),
            Commands::Grade => grade_logic(settings),
            Commands::Upload => upload_logic(settings),
            Commands::Download => download_logic(settings, arg),
            Commands::Template => template_logic(settings),
            Commands::Login => login_logic(settings)
        }
    }
}

fn open_logic(settings: Settings) -> () {
    let out_dir = get_current_attempt(settings.config.get("auth", "token").unwrap_or("".to_string())).path;
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
        .arg(get_current_attempt(settings.config.get("auth", "token").unwrap()).path)
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


fn download_logic(settings: Settings, arg: String) {
    let token = settings.config.get("auth", "token").unwrap_or("".to_string());
    let url_arg = format!("/api/attempts/@{}", arg.replace("~", ":"));
    let response = utils::request(url_arg, &token, "".to_string());

    let attempts = match response {
        Some(data) => utils::response_to_json(data),
        None => {
            eprintln!("no attempt found");
            exit(1)
        }
    };

    let attempt = &attempts[0];

    match attempt.as_object() {
        Some(select_attempt) => {
            let mut out_dir = get_lms_dir();

            match select_attempt.get("path") {
                Some(att) => {

                    out_dir.push(att.as_str().unwrap());
                    println!("{:?}", out_dir);

                    if Path::exists(&out_dir) {
                        eprintln!("output directory {} already exists", out_dir.to_str().unwrap());
                        exit(1)
                    }

                    let select_attempts = select_attempt.get("spec").unwrap().clone();

                    fs::create_dir_all(&out_dir);

                    let url = format!("/api/attempts/{}/submission", select_attempts.as_str().unwrap());
                    utils::download_tgz(url, &token, out_dir)
                }
                None => exit(1)
            }

        },

        None => {
            eprintln!("Cant find attempt");
            exit(1)
        }
    }
}

fn template_logic(settings: Settings) {
    let token = settings.config.get("auth", "token").unwrap_or("".to_string());
    let current_attempt = get_current_attempt(token.clone());

    download_template(token, current_attempt.id, current_attempt.path);
}

fn get_lms_dir() -> PathBuf {
    let mut lms_dir = PathBuf::new();
    lms_dir.push(env::var("HOME").unwrap());
    lms_dir.push("lms");

    lms_dir
}

fn get_current_attempt(token: String) -> Attempt {
    let mut lms_dir = get_lms_dir();

    let mut cache = lms_dir.clone();
    cache.push(".cache");

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
            let mut content = cache_location.split_whitespace();
            if let (Some(path), Some(spec), Some(id)) = (content.next(), content.next(), content.next()) {
                return Attempt::new(path.into(), spec.to_string(), id.to_string())
            } else {
                fs::remove_file(&cache);
            }
            
        } 

        eprintln!("No cache file");
        exit(1)
    }

    let online_attempt = utils::response_to_json(res.unwrap());
    let assignment_path = &online_attempt;

    let relative_path = &assignment_path.get("path").unwrap().as_str().unwrap();

    let id = &assignment_path.get("attempt_id").unwrap().as_number().unwrap();
    let spec = &assignment_path.get("spec").unwrap().as_str().unwrap();

    lms_dir.push(relative_path);
    let cache_value = format!("{} {} {}", &lms_dir.to_str().unwrap(), spec, &id);


    match fs::write(&cache, cache_value) {
        Ok(_) => {},
        Err(err) => eprintln!("Can't write to cache because: {}", err)
    }

    return Attempt::new(lms_dir, spec.to_string(), id.to_string())

}

fn download_template(token: String, attempt_id: String, out_dir: PathBuf) -> bool {
    if !Path::exists(&out_dir) {
        fs::create_dir_all(&out_dir);
    } else {
        return false
    }

    let url = format!("/api/attempts/{}/template", attempt_id);

    println!("{:?}", attempt_id);
    utils::download_tgz(url, &token, out_dir.clone());
    println!("Created {}", &out_dir.to_str().unwrap());
    return true
}
