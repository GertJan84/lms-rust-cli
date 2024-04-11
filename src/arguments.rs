use crate::{attempt::Attempt, files, io, prompt, settings::Settings, show};
use gethostname::gethostname;
use glob::glob;
use rand::{distributions::Alphanumeric, Rng};
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::{exit, Command, Stdio},
    thread::sleep,
    time::Duration,
};
use url::form_urlencoded;
use webbrowser;

const AUTH_TOKEN_LENGTH: u8 = 69;
const SCAN_FILE_TYPE: [&str; 7] = ["sql", "rs", "py", "js", "css", "html", "svelte"];
const DOWNLOAD_EXCLUDE: [&str; 3] = ["exam", "project", "graduation"];

pub fn execute(command: &str, arg: String) {
    let settings = Settings::new();
    match command {
        "open" => open_logic(&settings),
        "grade" => grade_logic(&settings, arg),
        "upload" => upload_logic(&settings),
        "download" => download_logic(&settings, arg),
        "template" => template_logic(&settings),
        "update" => io::handle_upgrade(),
        "verify" => verify_logic(),
        "login" => login_logic(settings),
        "show" => show::show(&settings, arg),
        _ => {
            eprintln!("invalid command {}", command);
        }
    }

    exit(1)
}


fn open_ide(path: &PathBuf, editors: &Vec<String>) -> () {
    if let Err(err) = env::set_current_dir(&path) {
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
            }
            Err(_) => {}
        }
    }

    for editor in &editors {
        let mut editor_parts = editor.split_whitespace();
        let editor_name = editor_parts.next().unwrap_or_default();
        let mut args: Vec<&str> = editor_parts.collect();

        if args.is_empty() {
            args.push(".");
        }

        // Skip to the next editor if the current one is not available
        if !io::is_installed(editor_name) {
            continue;
        }

        Command::new(editor_name)
            .args(args)
            .status()
            .expect("Failed to execute editor");
        exit(0);
    }
}

fn open_logic(settings: &Settings) -> () {
    let current_attempt = Attempt::get_current_attempt(&settings);

    if !download_template(&current_attempt.token, &current_attempt) {
        println!(
            "Already exists in {}",
            &current_attempt.get_path_buf().to_str().unwrap().to_string()
        );
    }

    if current_attempt.offline {
        open_ide(&current_attempt.get_path_buf(), &settings.editors)
    }

    // if settings.get_setting("setup", "move_node_directories", true) {
    //     verify_logic()
    // }

    open_ide(&current_attempt.get_path_buf(), &settings.editors)
}


fn grade_logic(settings: &Settings, arg: String) {
    let token = settings
        .config
        .get("auth", "token")
        .unwrap_or("".to_string());
    let url_arg = format!("/api/attempts/{}", arg.replace("~", ":"));
    let response = io::request("GET", url_arg, &token, None, true);

    let attempts = match response {
        Some(data) => io::response_to_json(data),
        None => {
            return eprintln!("No attempt found");
        }
    };

    let attempt = &attempts[0];

    let out_dir = files::get_lms_dir().join("grading").join(
        attempt
            .get("spec")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
            .replace(":", "~"),
    );

    if Path::exists(&out_dir) {
        if files::is_folder_empty(&out_dir).unwrap() {
            match fs::remove_dir_all(&out_dir) {
                Ok(_) => {}
                Err(err) => eprintln!("Cant remove directory because: {}", err),
            }
        }
    }

    if Path::exists(&out_dir) {
        eprintln!(
            "Submission already exists in {}",
            out_dir.to_str().unwrap().to_string()
        )
    } else {
        let _ = fs::create_dir_all(&out_dir);
        let url = format!(
            "/api/attempts/{}/submission",
            attempt.get("spec").unwrap().as_str().unwrap().to_string()
        );
        io::download_tgz(url, &token, &out_dir);
        println!("Downloaded to {}", out_dir.to_str().unwrap().to_string());
    }

    for name in vec!["_node", "_solution", "_template"] {
        let _ = fs::remove_dir_all(&out_dir.join(name));

        let mut curriculum_dir = PathBuf::new();
        curriculum_dir.push(env::var("HOME").unwrap());
        curriculum_dir.push(
            settings
                .config
                .get("grade", "curriculum_directory")
                .unwrap_or("curriculum".to_string()),
        );

        let mut glob_path = PathBuf::new();
        glob_path.push(&curriculum_dir);
        glob_path.push(&attempt.get("period").unwrap().to_string());
        glob_path.push(&attempt.get("module_id").unwrap().to_string());
        glob_path.push(format!(
            "[0-9][0-9]-{}",
            &attempt.get("node_id").unwrap().to_string()
        ));

        let glob_str = glob_path.to_str().expect("Invalid UTF-8 in path");
        if let Ok(mut paths) = glob(&glob_str) {
            match paths.next() {
                Some(found_node_id) => {
                    let node_id = found_node_id.unwrap();
                    let _ = symlink(
                        &node_id,
                        out_dir.join(format!("_{}", node_id.to_str().unwrap().to_string())),
                    );

                    for what in vec!["solution", "template"] {
                        let what_dir = out_dir.join(format!(
                            "{}{}",
                            what,
                            attempt
                                .get("variant_id")
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_string()
                        ));
                        if let Ok(metadata) = fs::metadata(&what_dir) {
                            let _ = metadata
                                .is_dir()
                                .then(|| symlink(&what, out_dir.join(format!("_{}", what))))
                                .expect("Failed to create symlink");
                        };
                    }
                }
                None => {}
            }
        }
    }

    open_ide(&out_dir, &settings.editors)
}

fn login_logic(mut settings: Settings) {
    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(AUTH_TOKEN_LENGTH.into())
        .map(char::from)
        .collect();

    settings.set("auth".to_string(), "token".to_string(), token.clone());
    let encoded_host =
        form_urlencoded::byte_serialize(gethostname().as_encoded_bytes()).collect::<String>();
    let url = format!(
        "{}/api/authorize?host={}&token={}",
        crate::BASE_URL.to_string(),
        encoded_host,
        &token
    );
    println!("Go to this URL to authorize lms: {}", url);
    let _ = webbrowser::open(url.as_str());
}

fn upload_logic(settings: &Settings) {
    let current_attempt = Attempt::get_current_attempt(settings);
    
    if !Path::exists(&current_attempt.get_path_buf()) {
        eprintln!(
            "There is no folder: {}",
            current_attempt.get_path_buf().to_str().unwrap()
        );
        return eprintln!("Try `lms template` first");
    }

    if settings
        .config
        .getbool("custom", "check_todo")
        .unwrap()
        .unwrap_or(true)
    {
        if let Some(file_todo) = get_todo(&current_attempt.get_path_buf()) {
            println!("You still have some TODO's in your code: ");
            for (file, todos) in file_todo {
                println!("\n{}: has some TODO's:", file);

                for (idx, line) in todos {
                    println!("  {} -> {}", idx, line)
                }
            }

            if prompt::yes_no("\nYou still have some TODO's in your code do you want to fix them") {
                return println!("Upload cancelled");
            }
        }
    }

    let cmd = if cfg!(target_os = "macos") {
        "gtar"
    } else {
        "tar"
    };

    if files::is_folder_empty(&current_attempt.get_path_buf()).unwrap() {
        if !prompt::yes_no("This folder is currently empty are you sure you want to upload?") {
            return eprintln!("Cancelled upload");
        }
    }

    let mut tar = Command::new(cmd);
    tar.arg("czC")
        .arg(current_attempt.get_path_buf().to_str().unwrap().to_string())
        .arg("--exclude-backups")
        .arg("--exclude-ignore=.gitignore")
        .arg("--exclude-ignore=.lmsignore")
        .arg(".")
        .stdin(Stdio::null())
        .stdout(Stdio::piped());

    let data = match tar.output() {
        Ok(output) => output,
        Err(_) => {
            if cfg!(platform = "macos") {
                return println!("Please install gnu-tar (using brew for instance)");
            }
            return eprintln!("Command not found: {}", cmd);
        }
    };

    let url = format!(
        "/api/attempts/{}/submission",
        current_attempt.id.to_string()
    );

    match io::request("POST", url, &current_attempt.token, Some(data.stdout), true) {
        Some(res) => {
            let json_res: serde_json::Value = io::response_to_json(res);

            match json_res.get("transferred") {
                Some(transferred) => {
                    if let Some(upload_bytes) = transferred.as_u64() {
                        let upload_kb = upload_bytes / 1024;
                        println!("Uploaded complete: {}kb transferred", upload_kb);

                        if settings.get_setting("setup", "upload_open_browser", true) {
                            let _ = webbrowser::open(&current_attempt.get_url());
                        }
                        else {
                            println!("Please remember that you still need to submit in the web interface");
                        }

                    }
                }
                None => {
                    return eprintln!("Error getting transferred value");
                }
            }
        }
        None => {
            return eprintln!("Failed to upload attempt");
        }
    }
}

fn download_logic(settings: &Settings, arg: String) {
    let token = settings
        .config
        .get("auth", "token")
        .unwrap_or("".to_string());

    if !arg.eq("all") {
        let _ = download_attempt(&arg, &token);
    }

    let response = io::request("GET", "/api/node-paths".to_string(), &token, None, true);
    let attempts = match response {
        Some(data) => io::response_to_json(data),
        None => {
            return eprintln!("No attempt found");
        }
    };

    let mut local_dirs: HashSet<String> = HashSet::new();
    let target_dir = files::get_lms_dir().join("*/*");
    for path in glob(target_dir.to_str().unwrap()).expect("Failed to read lms dir") {
        match path {
            Ok(path) => {
                local_dirs.insert(
                    path.as_path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                );
            }
            Err(_) => {}
        }
    }

    attempts
        .as_object()
        .unwrap()
        .iter()
        .for_each(|(assignment, _)| {
            let mut ignore = false;

            for exclude in DOWNLOAD_EXCLUDE {
                if assignment.contains(exclude) {
                    ignore = true;
                    break;
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
    let response = io::request("GET", url_arg, token, None, true);

    let attempts = match response {
        Some(data) => io::response_to_json(data),
        None => {
            eprintln!("No attempt found: {}", assignment);
            return false;
        }
    };

    let attempt = &attempts[0];

    match attempt.as_object() {
        Some(select_attempt) => {
            let mut out_dir = files::get_lms_dir();

            match select_attempt.get("path") {
                Some(att) => {
                    out_dir.push(att.as_str().unwrap());

                    if Path::exists(&out_dir) {
                        eprintln!(
                            "Output directory {} already exists",
                            out_dir.to_str().unwrap()
                        );
                        return false;
                    }

                    let select_attempts = select_attempt.get("spec").unwrap().clone();

                    let _ = fs::create_dir_all(&out_dir);

                    let url = format!(
                        "/api/attempts/{}/submission",
                        select_attempts.as_str().unwrap()
                    );
                    io::download_tgz(url, &token, &out_dir);
                    println!(
                        "Downloaded: {} at: {}",
                        assignment,
                        &out_dir.to_str().unwrap()
                    );
                }
                None => return false,
            }
        }

        None => {
            eprintln!("Cant find attempt: {}", assignment);
            return false;
        }
    }
    return true;
}

fn template_logic(settings: &Settings) {
    let current_attempt = Attempt::get_current_attempt(settings);

    if !download_template(&current_attempt.token, &current_attempt) {
        let error_message = format!(
            "Output directory {} already exists",
            current_attempt.get_path_buf().to_str().unwrap().to_string()
        );

        return eprintln!("{}", error_message);
    }
}

fn download_template(token: &String, attempt: &Attempt) -> bool {
    if !Path::exists(&attempt.get_path_buf()) {
        let _ = fs::create_dir_all(&attempt.get_path_buf());
        println!("Created {}", &attempt.get_path_buf().to_str().unwrap());
    } else {
        if !files::is_folder_empty(&attempt.get_path_buf()).unwrap() {
            return false;
        }
    }

    if attempt.offline {
        println!("No connection to server");
        return false;
    }

    let url = format!("/api/attempts/{}/template", &attempt.id);
    io::download_tgz(url, &token, &attempt.get_path_buf());
    true
}

fn verify_logic() {
    let misplaced = files::get_misplaced_nodes();
    if misplaced.len() != 0 {
        println!("These directories are not in their recommended locations:");
        for (local_directory, valid_directory) in &misplaced {
            println!(
                "  {} -> {}",
                local_directory.to_str().unwrap().to_string(),
                valid_directory.to_str().unwrap().to_string()
            );
        }

        if !prompt::yes_no("Would you like to move them?") {
            return;
        }

        // If you want to replace them
        for (local_directory, valid_directory) in &misplaced {
            let _ = fs::rename(local_directory, valid_directory);
        }
    }
    println!("All nodes are in the right place!");
    }

fn get_todo(project_folder: &PathBuf) -> Option<HashMap<String, HashMap<usize, String>>> {
    let mut file_todo = HashMap::new();

    for files in glob(project_folder.join("*").to_str().unwrap()).unwrap() {
        if let Ok(file) = files {
            if !file.is_file() {
                continue;
            }

            match file.extension() {
                Some(ext) => {
                    if !SCAN_FILE_TYPE.contains(&ext.to_str().unwrap()) {
                        continue;
                    }
                }
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
                file_todo.insert(
                    file.file_name().unwrap().to_str().unwrap().to_string(),
                    todo_dict,
                );
            }
        }
    }

    if file_todo.len() != 0 {
        return Some(file_todo);
    }

    None
}

