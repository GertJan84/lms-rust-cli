use std::path::{Path, PathBuf};
use glob::glob;
use std::env;
use std::fs;
use std::process::{Command, exit, Stdio};
use rand::{Rng, distributions::Alphanumeric};
use gethostname::gethostname;
use url::form_urlencoded;
use webbrowser;
use std::collections::HashMap;
use crate::{settings::Settings, utils};

const AUTH_TOKEN_LENGHT: u8 = 69;

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

pub fn execute(command: &str, arg: String) {
    let settings = Settings::new();
    match command {
        "open" => open_logic(settings),
        "grade" => grade_logic(settings),
        "upload" => upload_logic(settings),
        "download" => download_logic(settings, arg),
        "template" => template_logic(settings),
        "verify" => handle_verify(),
        "login"=> login_logic(settings),
        _ => {
            eprintln!("invalid command {}", command);
            exit(1)
        }
    }
}

fn open_logic(settings: Settings) -> () {
    let token = settings.config.get("auth", "token").unwrap_or("".to_string());
    let current_attempt = get_current_attempt(token.clone());

    if !download_template(token, &current_attempt) {
        println!("Already exists in {}", current_attempt.path.to_str().unwrap().to_string());
    }

    match env::set_current_dir(current_attempt.path) {
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
    let cmd = if cfg!(target_os = "macos") {
        "gtar"
    } else {
        "tar"
    };

    let token = settings.config.get("auth", "token").unwrap();
    let current_attempt = get_current_attempt(token.clone());

    // TODO: Check if something is in the folder
    if !Path::exists(&current_attempt.path) {
        eprintln!("There is no folder: {}", current_attempt.path.to_str().unwrap());
        eprintln!("Try `lms template` first");
        exit(1)
    }
    
    let mut tar = Command::new(cmd);
    tar.arg("czC")
        .arg(current_attempt.path.to_str().unwrap().to_string())
        .arg("--exclude-backups")
        .arg("--exclude-ignore=.gitignore")
        .arg("--exclude-ignore=.lmsignore")
        .arg(".")
        .stdout(Stdio::piped());

    let data = match tar.output() {
        Ok(output) => output,
        Err(_) => {
            eprintln!("Command not found: {}", cmd);
            if cfg!(platform = "macos") {
                println!("Please install gnu-tar (using brew for instanse")
            }
            exit(1)
        }
    };

    let url = format!("/api/attempts/{}/submission", current_attempt.id.to_string());

    match utils::request("POST", url, &token, Some(data.stdout)) {
        Some(res) => {
            let json_res: serde_json::Value = utils::response_to_json(res);

            match json_res.get("transferred") {
                Some(transferred) => {
                    if let Some(upload_bytes) = transferred.as_u64() {
                        let upload_kb = upload_bytes / 1024;
                        println!("Uploaded complete: {}kb transferred", upload_kb);
                        println!("Please remember that you still need to submit in the web interface")
                    }
                },
                None => {
                    eprintln!("Error getting transferred value");
                    exit(1)
                }
            }
        },
        None => {
            eprintln!("Faild to upload attempt");
            exit(1)
        }
    }
}


fn download_logic(settings: Settings, arg: String) {
    let token = settings.config.get("auth", "token").unwrap_or("".to_string());
    let url_arg = format!("/api/attempts/@{}", arg.replace("~", ":"));
    let response = utils::request("GET", url_arg, &token, None);

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

                    let _ = fs::create_dir_all(&out_dir);

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

   if !download_template(token, &current_attempt) {
        // TODO: Add path of folder to error message
        println!("output directory already exists");
    }
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

    let res = utils::request("GET", "/api/attempts/current".to_string(), &token, None);

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
            } 

            let _ = fs::remove_file(&cache);
            
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

    Attempt::new(lms_dir, spec.to_string(), id.to_string())

}

fn download_template(token: String, attempt: &Attempt) -> bool {
    if !Path::exists(&attempt.path) {
        let _ = fs::create_dir_all(&attempt.path);
    } else {
        return false
    }

    let url = format!("/api/attempts/{}/template", &attempt.id);

    utils::download_tgz(url, &token, attempt.path.clone());
    println!("Created {}", &attempt.path.to_str().unwrap());
    true
}

fn handle_verify() {
    move_node_directories();
}

fn move_node_directories() -> Result<(), std::io::Error> {
    let lms_dir = get_lms_dir();

    let correct_pathes_json = match utils::request("GET", "/api/node-paths".to_string(), &"".to_string(), None) {
        Some(data) => utils::response_to_json(data),
        None => {
            eprintln!("Cant convert paths to json");
            exit(1)
        }
    };

    let misplaced: HashMap<PathBuf, PathBuf> = HashMap::new();
    
    println!("{:?}", correct_pathes_json);

    let target_dir = lms_dir.join("*/*");
    // Get all directorys in lms [python, pwa, static-web, ..etc]
    for dir in glob(target_dir.to_str().unwrap()).expect("Faild to read lms dir") {

        let local_path = dir.as_ref().unwrap().parent().unwrap().file_name().unwrap();

        // Get all chilled directorys in lms [css, vars, svelte, ..etc]
        if let Ok(ref path) = dir {
            if path.is_dir() {
                let node_id = path.file_name().unwrap();
                if local_path.eq("grading") {
                    continue
                }
                println!("node_id: {:?}, local: {:?}", node_id, local_path);
            }
        }
    }

    Ok(())
}
