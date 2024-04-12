use configparser::ini::WriteOptions;

use crate::{attempt::Attempt, settings::Settings};

pub fn show(settings: &Settings, arg: String) {
    let arg = arg.as_str();

    match arg {
        "path" => show_path(settings),
        "settings" => show_settings(settings),
        _ => {
            eprintln!("invalid subcommand {}", arg);
        }
    }
}

fn show_path(settings: &Settings) {
    let current_attempt = Attempt::get_current_attempt(settings);

    let binding = current_attempt.get_path_buf();
    let path_str = &binding.to_str().unwrap_or("");
    println!("{}", path_str);
}

fn show_settings(settings: &Settings) {
    let binding = settings.config.pretty_writes(&WriteOptions::default());
    let settings_str = binding.as_str();
    println!("{}", settings_str);
}
