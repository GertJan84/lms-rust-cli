use clap::builder::Str;
use glob::glob;
use std::{
    process::{Command, exit, Stdio}, 
    thread::sleep,
    time::Duration,
    collections::{HashSet, HashMap},
    path::{Path, PathBuf},
    os::unix::fs::symlink,
    fs,
    env
};
use rand::{Rng, distributions::Alphanumeric};
use gethostname::gethostname;
use url::form_urlencoded;
use webbrowser;
use crate::{settings::Settings, utils};

const AUTH_TOKEN_LENGHT: u8 = 69;
const SCAN_FILE_TYPE: [&str; 7] = ["sql", "rs", "py", "js", "css", "html", "svelte"];
const DOWNLOAD_EXCLUDE: [&str; 3] = ["exam", "project", "graduation"];

struct Attempt {
    path: PathBuf,
    spec: String,
    id: String,
    offline: bool
}

impl Attempt {
    pub fn new(path: PathBuf, spec: String, id: String, offline: bool) -> Self {
        Self {
            path,
            spec,
            id,
            offline
        }
    }
}

pub fn execute(command: &str, arg: String) {
    let settings = Settings::new();
    match command {
        "open" => open_logic(&settings),
        "grade" => grade_logic(&settings, arg),
        "upload" => upload_logic(&settings),
        "download" => download_logic(&settings, arg),
        "template" => template_logic(&settings),
        "install" => install_logic(),
        "verify" => verify_logic(),
        "login"=> login_logic(settings),
        _ => {
            eprintln!("invalid command {}", command);
            exit(1)
        }
    }
}

fn open_ide(current_attempt: &Attempt, editors: &Vec<String>) -> () {
    if let Err(err) = env::set_current_dir(&current_attempt.path) {
        eprintln!("{}", err);
        return;
    }

    let mut editors = editors.clone();

    if Path::new(".lms-ide").exists() {
        match fs::read_to_string(".lms-ide") {
            Ok(lms_ide) => {
                // Parse lms_ide file to exclude dots and remove white so that "android-studio . " becomes "android-studio"
                let lms_ide = lms_ide
                    .split_whitespace()
                    .filter(|&x| !x.contains("."))
                    .collect();

                editors.insert(0, lms_ide)
            },
            Err(_) => {},
        }
    }

    for editor in &editors {
        let mut editor_parts = editor.split_whitespace();
        let editor_name = editor_parts.next().unwrap_or_default();
        let mut args: Vec<&str> = editor_parts.collect();

        if args.is_empty() {
            args.push(".");
        }

        // Check if the editor is available in the system's PATH
        let editor_available = Command::new("which")
            .arg(editor)
            .stderr(Stdio::null())
            .status()
            .expect("Can't find which")
            .success();

        // Skip to the next editor if the current one is not available
        if !editor_available {
            continue;
        }

        // Attempt to launch the editor and exit on success
        Command::new(editor_name)
            .args(args)
            .status()
            .expect("Failed to execute editor");
        exit(0);
    }

    
}

fn open_logic(settings: &Settings) -> () {
    let token = settings.config.get("auth", "token").unwrap_or("".to_string());
    let current_attempt = get_current_attempt(token.clone());
    
    if !download_template(token, &current_attempt) {
        println!("Already exists in {}", &current_attempt.path.to_str().unwrap().to_string());
    }

    if current_attempt.offline {
        open_ide(&current_attempt, &settings.editors)
    }


    if settings.config.getbool("setup", "move_node_directories").unwrap().unwrap_or(true) {
        verify_logic()
    }

    open_ide(&current_attempt, &settings.editors)
}

fn install_logic() {
    eprintln!("This feature is used for the recommanded Vistual studio code setup.");
    eprintln!("This feature not implemented.");
    exit(0)
}

fn grade_logic(settings: &Settings, arg: String) {
    let token = settings.config.get("auth", "token").unwrap_or("".to_string());
    let url_arg = format!("/api/attempts/{}", arg.replace("~", ":"));
    let response = utils::request("GET", url_arg, &token, None);

    let attempts = match response {
        Some(data) => utils::response_to_json(data),
        None => {
            eprintln!("no attempt found");
            exit(1)
        }
    };

    let attempt = &attempts[0];

    let out_dir = utils::get_lms_dir().join("grading").join(attempt.get("spec").unwrap().as_str().unwrap().to_string().replace(":", "~"));

    if Path::exists(&out_dir) {
        if utils::is_folder_empty(&out_dir).unwrap() {
            match fs::remove_dir_all(&out_dir) {
                Ok(_) => {},
                Err(err) => eprintln!("Cant remove directory because: {}", err)
            }
        }
    }
   
    if Path::exists(&out_dir) {
        eprintln!("Subbmission already exsists in {}", out_dir.to_str().unwrap().to_string())
    } else {
        let _ = fs::create_dir_all(&out_dir);
        let url = format!("/api/attempts/{}/submission", attempt.get("spec").unwrap().as_str().unwrap().to_string());
        utils::download_tgz(url, &token, &out_dir);
        println!("Downloaded to {}", out_dir.to_str().unwrap().to_string());
    }

    for name in vec!["_node", "_solution", "_template"] {
        let _ = fs::remove_dir_all(&out_dir.join(name));


        let mut curruculum_dir = PathBuf::new();
        curruculum_dir.push(env::var("HOME").unwrap());
        curruculum_dir.push(settings.config.get("grade", "curriculum_directory").unwrap_or("curriculum".to_string()));

        let mut glob_path = PathBuf::new();
        glob_path.push(&curruculum_dir);
        glob_path.push(&attempt.get("period").unwrap().to_string());
        glob_path.push(&attempt.get("module_id").unwrap().to_string());
        glob_path.push(format!("[0-9][0-9]-{}", &attempt.get("node_id").unwrap().to_string()));

        let glob_str = glob_path.to_str().expect("Invalid UTF-8 in path");
        if let Ok(mut paths) = glob(&glob_str) {
            match paths.next() {
                Some(found_node_id) => {
                    let node_id = found_node_id.unwrap();
                    let _ = symlink(&node_id, out_dir.join(format!("_{}", node_id.to_str().unwrap().to_string())));

                    for what in vec!["solution", "template"] {
                        let what_dir = out_dir.join(format!("{}{}", what, attempt.get("variant_id").unwrap().as_str().unwrap().to_string()));
                        if let Ok(metadata) = fs::metadata(&what_dir) {
                            let _ = metadata
                                .is_dir()
                                .then(|| symlink(&what, out_dir.join(format!("_{}", what))))
                                .expect("Faild to create symlink");
                        };
                    }
                },
                None => {} 
            }
        }
    }
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

fn upload_logic(settings: &Settings) {

    let token = settings.config.get("auth", "token").unwrap();
    let current_attempt = get_current_attempt(token.clone());

    if !Path::exists(&current_attempt.path) {
        eprintln!("There is no folder: {}", current_attempt.path.to_str().unwrap());
        eprintln!("Try `lms template` first");
        exit(1)
    }
    
    if settings.config.getbool("custom", "check_todo").unwrap().unwrap_or(true) {
        if let Some(file_todo) = get_todo(&current_attempt.path) {
            println!("You still have some todo in your code: ");
            for (file, todos) in file_todo {
                println!("\n{}: has some todos:", file);

                for (idx, line) in todos {
                    println!("  {} -> {}", idx, line)
                }

            }

            if utils::prompt_yes_no("\nYou still have some TODO'S in your code do you want to fix them") {
                println!("Upload cancelled");
                exit(0)
            }
        }

    }


    let cmd = if cfg!(target_os = "macos") {
        "gtar"
    } else {
        "tar"
    };



    if utils::is_folder_empty(&current_attempt.path).unwrap() {
        if !utils::prompt_yes_no("This folder is currently empty are you sure you want to upload") {
            exit(0)
        }
    }
    
    let mut tar = Command::new(cmd);
    tar.arg("czC")
        .arg(current_attempt.path.to_str().unwrap().to_string())
        .arg("--exclude-backups")
        .arg("--exclude-ignore=.gitignore")
        .arg("--exclude-ignore=.lmsignore")
        .arg(".")
        .stdin(Stdio::null())
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


fn download_logic(settings: &Settings, arg: String) {
    let token = settings.config.get("auth", "token").unwrap_or("".to_string());

    if !arg.eq("all") {
        let _  = download_attempt(&arg, &token);
    }

    let response = utils::request("GET", "/api/node-paths".to_string(), &token, None);
    let attempts = match response {
        Some(data) => utils::response_to_json(data),
        None => {
            eprintln!("no attempt found");
            exit(1)
        }
    };

    let mut local_dirs: HashSet<String> = HashSet::new();
    let target_dir = utils::get_lms_dir().join("*/*");
    for path in  glob(target_dir.to_str().unwrap()).expect("Faild to read lms dir") {
        match path {
            Ok(path) => {
                local_dirs.insert(path.as_path().file_name().unwrap().to_str().unwrap().to_string());
            },
            Err(_) => {}
        }
    }

    attempts.as_object().unwrap().iter().for_each(|(assignment, _)| {
        let mut ignore = false;

        for exclude in DOWNLOAD_EXCLUDE {
            if assignment.contains(exclude) {
                ignore = true;
                break
            }
        }

        if !local_dirs.contains(assignment) && !ignore {
            download_attempt(&assignment.to_string(), &token);
            sleep(Duration::from_millis(500));
        }
    })
}


fn download_attempt(assignment: &String, token: &String) -> bool {
    let url_arg = format!("/api/attempts/@{}", assignment.replace("~", ":"));
    let response = utils::request("GET", url_arg, token, None);

    let attempts = match response {
        Some(data) => utils::response_to_json(data),
        None => {
            eprintln!("no attempt found: {}", assignment);
            return false
        }
    };

    let attempt = &attempts[0];

    match attempt.as_object() {
        Some(select_attempt) => {
            let mut out_dir = utils::get_lms_dir();

            match select_attempt.get("path") {
                Some(att) => {

                    out_dir.push(att.as_str().unwrap());

                    if Path::exists(&out_dir) {
                        eprintln!("output directory {} already exists", out_dir.to_str().unwrap());
                        return false
                    }

                    let select_attempts = select_attempt.get("spec").unwrap().clone();

                    let _ = fs::create_dir_all(&out_dir);

                    let url = format!("/api/attempts/{}/submission", select_attempts.as_str().unwrap());
                    utils::download_tgz(url, &token, &out_dir);
                    println!("Downloaded: {} at: {}", assignment, &out_dir.to_str().unwrap());
                }
                None => return false 
            }

        },

        None => {
            eprintln!("Cant find attempt: {}", assignment);
            return false
        }
    }
    return true
}

fn template_logic(settings: &Settings) {
    let token = settings.config.get("auth", "token").unwrap_or("".to_string());
    let current_attempt = get_current_attempt(token.clone());

   if !download_template(token, &current_attempt) {
        let error_message = format!("output directory {} already exists", current_attempt.path.to_str().unwrap().to_string());
        eprintln!("{}", error_message);
        exit(1)
    }
}


fn get_current_attempt(token: String) -> Attempt {
    let mut lms_dir = utils::get_lms_dir();

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
                return Attempt::new(path.into(), spec.to_string(), id.to_string(), true)
            } 
            let _ = fs::remove_file(&cache);
        } 
        eprintln!("No cache file");
        exit(1)
    }

    let online_attempt = utils::response_to_json(res.unwrap());
    let assignment_path = &online_attempt;

    if assignment_path.is_null() {
        println!("You currently dont have a assingment open");  
        exit(0)
    }

    let relative_path = &assignment_path.get("path").unwrap().as_str().unwrap();

    let id = &assignment_path.get("attempt_id").unwrap().as_number().unwrap();
    let spec = &assignment_path.get("spec").unwrap().as_str().unwrap();

    lms_dir.push(relative_path);
    let cache_value = format!("{} {} {}", &lms_dir.to_str().unwrap(), spec, &id);


    match fs::write(&cache, cache_value) {
        Ok(_) => {},
        Err(err) => eprintln!("Can't write to cache because: {}", err)
    }

    Attempt::new(lms_dir, spec.to_string(), id.to_string(), false)

}

fn download_template(token: String, attempt: &Attempt) -> bool {
    if !Path::exists(&attempt.path) {
        let _ = fs::create_dir_all(&attempt.path);
        println!("Created {}", &attempt.path.to_str().unwrap());
    } else {
        if !utils::is_folder_empty(&attempt.path).unwrap() {
            return false
        }
    }

    if attempt.offline {
        println!("No connection to server");
        return false
    }

    let url = format!("/api/attempts/{}/template", &attempt.id);
    utils::download_tgz(url, &token, &attempt.path);
    true
}

fn verify_logic() {
    if move_node_directories() {
        println!("All nodes are in the right place!");
    }
}

fn move_node_directories() -> bool {
    let lms_dir = utils::get_lms_dir();

    let correct_pathes_json = match utils::request("GET", "/api/node-paths".to_string(), &"".to_string(), None) {
        Some(data) => utils::response_to_json(data),
        None => {
            eprintln!("Cant convert paths to json");
            exit(1)
        }
    };

    let mut misplaced: HashMap<PathBuf, PathBuf> = HashMap::new();
    
    let target_dir = lms_dir.join("*/*");
    // Get all directorys in lms [python, pwa, static-web, ..etc]
    for dir in glob(target_dir.to_str().unwrap()).expect("Faild to read lms dir") {

        let local_path_current = dir.as_ref().unwrap().parent().unwrap().file_name().unwrap();

        // Get all chilled directorys in lms [css, vars, svelte, ..etc]
        if let Ok(ref path) = dir {
            if path.is_dir() {
                let node_id = path.file_name().unwrap().to_str().unwrap().to_string();

                if local_path_current.eq("grading") {
                    continue
                }

                // TODO: Refactor this
                if let correct_path_object = Some(&correct_pathes_json) {
                    let pressent_node_id = correct_pathes_json.as_object().unwrap().get(&node_id);
                    if pressent_node_id.is_some() {
                        if let correct_path = pressent_node_id.unwrap().as_str().unwrap().to_string() {

                            if !correct_path.eq(local_path_current.to_str().unwrap()) {
                                let local_path = lms_dir.join(local_path_current).join(&node_id);
                                let valid_path = lms_dir.join(correct_path).join(&node_id);

                                if !Path::exists(&valid_path) {
                                    misplaced.insert(
                                        local_path,
                                        valid_path
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if misplaced.len() != 0 {
        println!("These directories are not in their recommanded locations:");
        for (local_directory, valid_directory) in &misplaced {
            println!("  {} -> {}", local_directory.to_str().unwrap().to_string(), valid_directory.to_str().unwrap().to_string());
            let permission = utils::prompt_yes_no("Would you like to move them");

            if !permission {
                return false 
            }
            let _ = fs::rename(local_directory, valid_directory);
        }
    }
    true
}

fn get_todo(project_folder: &PathBuf) -> Option<HashMap<String, HashMap<usize, String>>> {
    
    let mut file_todo = HashMap::new();

    for files in  glob(project_folder.join("*").to_str().unwrap()).unwrap() {
        if let Ok(file) = files {

            if !file.is_file() {
                continue
            } 

            match file.extension() {
                Some(ext) => {
                    if !SCAN_FILE_TYPE.contains(&ext.to_str().unwrap()) {
                        continue
                    }
                },
                None => continue,
            }


            let lines: Vec<String> = fs::read_to_string(&file)
                .unwrap()
                .lines()
                .map(String::from)
                .collect();


            let mut todo_dict = HashMap::new();
            lines.iter().enumerate().rev().for_each(|(idx, line)| {
                if line.contains("TODO") {
                    todo_dict.insert(idx + 1, line.to_string());
                }
            });

            if !todo_dict.is_empty() {
                file_todo.insert(file.file_name().unwrap().to_str().unwrap().to_string(), todo_dict);
            }
        }
    }

    if file_todo.len() != 0 {
        return Some(file_todo)
    }

    None

}

