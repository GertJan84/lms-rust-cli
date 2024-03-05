use std::path::{Path, PathBuf};
use std::env;
use std::process::Command;
use rand::{Rng, distributions::Alphanumeric};
use reqwest;
use gethostname::gethostname;
use crate::Settings;

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
    let out_dir = get_work_location();
    if let Err(err) = env::set_current_dir(out_dir) {
        eprintln!("{}", err)
    } else {
       Command::new("nvim").arg(".").status().expect("cant run editor");
    }
}

fn grade_logic(settings: Settings) {
    todo!()
}

fn login_logic(settings: Settings) {
    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(AUTH_TOKEN_LENGHT.into())
        .map(char::from)
        .collect();
    println!("{}", token);

    println!("{:?}", gethostname());
}

fn upload_logic(settings: Settings) {
    println!("{}", std::env::consts::OS);
}


fn download_logic(settings: Settings) {
    todo!()
}

fn template_logic(settings: Settings) {
    todo!()
}

fn get_work_location() -> PathBuf {

    let mut lms_dir = PathBuf::new();
    lms_dir.push(env::var("HOME").unwrap());
    lms_dir.push("lms");

    let mut cache = lms_dir.clone();
    cache.push(".cache");

    if Path::exists(&cache) {
        todo!("read cache file")
    } 

    lms_dir
    //reqwest
}


