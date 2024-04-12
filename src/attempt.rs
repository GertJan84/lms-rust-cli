use std::{fs, path::{Path, PathBuf}, process::exit};

use crate::{files, io, settings::Settings};

pub struct Attempt {
    pub node_id: String,
    pub module_id: String,
    pub spec: String,
    pub id: String,
    pub offline: bool,
    pub token: String
}

impl Attempt {
    pub fn new(
         node_id: String,
         module_id: String,
         spec: String, 
         id: String, 
         offline: bool, 
         token: String) -> Self {
        Self {
            node_id,
            module_id,
            spec,
            id,
            offline,
            token
        }
    }

    pub fn get_url(&self) -> String {
        format!("{}/{}/{}", crate::BASE_URL.to_string(), "curriculum".to_string(), self.node_id)
    }

    pub fn get_path_buf(&self) -> PathBuf {        
        PathBuf::from_iter(vec![files::get_lms_dir(), self.module_id.clone().into(), self.node_id.clone().into()])
    }

    pub fn get_current_attempt(settings: &Settings) -> Attempt {
        let token = settings.config.get("auth", "token").unwrap_or("".to_string());

        let lms_dir = files::get_lms_dir();
    
        let mut cache = lms_dir.clone();
        cache.push(".cache");
    
        let res = io::request("GET", "/api/attempts/current".to_string(), &token, None);
        
        if res.is_none() {
            return Self::get_offline_attempt(cache, token);
        }
       
        let online_attempt = io::response_to_json(res.unwrap());
        let assignment_path = &online_attempt;

        if assignment_path.is_null() {
            println!("You currently don't have an assignment open");  
            exit(0)
        }
    
        let relative_path = &assignment_path.get("path").unwrap().as_str().unwrap();

        let module_id = &assignment_path.get("module_id").unwrap().as_str().unwrap();
        let node_id = &assignment_path.get("node_id").unwrap().as_str().unwrap();

        let id = &assignment_path.get("attempt_id").unwrap().as_number().unwrap();
        let spec = &assignment_path.get("spec").unwrap().as_str().unwrap();
    
        let cache_value = format!("{} {} {}", &relative_path, spec, &id);
    
        match fs::write(&cache, cache_value) {
            Ok(_) => {},
            Err(err) => eprintln!("Can't write to cache because: {}", err)
        }

        Attempt::new(node_id.to_string(),module_id.to_string(), spec.to_string(), id.to_string(), false, token)
    
    }

    fn get_offline_attempt(cache: PathBuf, token: String) -> Attempt {
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

                let path_parts: Vec<&str> = path.split('/').collect();
                let module_id = path_parts[0];
                let node_id = path_parts[1];

                return Attempt::new(node_id.to_string(),module_id.to_string(), spec.to_string(), id.to_string(), true, token)
            }

            let _ = fs::remove_file(&cache);
        } 
        eprintln!("No cache file");
        exit(1)
    
    }
}


