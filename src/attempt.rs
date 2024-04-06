use std::{fs, path::{Path, PathBuf}, process::exit};

use crate::{files, io, settings::{self, Settings}};

pub struct Attempt {
    pub path: PathBuf,
    pub spec: String,
    pub id: String,
    pub offline: bool,
    pub token: String
}

impl Attempt {
    pub fn new(path: PathBuf, spec: String, id: String, offline: bool, token: String) -> Self {
        Self {
            path,
            spec,
            id,
            offline,
            token
        }
    }


    pub fn get_current_attempt(settings: &Settings) -> Attempt {
        let token = settings.config.get("auth", "token").unwrap_or("".to_string());

        let mut lms_dir = files::get_lms_dir();
    
        let mut cache = lms_dir.clone();
        cache.push(".cache");
    
        let res = io::request("GET", "/api/attempts/current".to_string(), &token, None, true);
    
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
                    return Attempt::new(path.into(), spec.to_string(), id.to_string(), true, token)
                } 
                let _ = fs::remove_file(&cache);
            } 
            eprintln!("No cache file");
            exit(1)
        }
    
        let online_attempt = io::response_to_json(res.unwrap());
        let assignment_path = &online_attempt;
    
        if assignment_path.is_null() {
            println!("You currently don't have an assignment open");  
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
    
        Attempt::new(lms_dir, spec.to_string(), id.to_string(), false, token)
    
    }
}


