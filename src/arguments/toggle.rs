use clap::Command;

use crate::{command_about, settings::Settings};

pub fn toggle_commands() -> Vec<Command> {
    vec![
        command_about!(
            "move_node_directories",
            "update your file structure so it matches correct"
        )
        .short_flag('D'),
        command_about!(
            "upload_open_browser",
            "upload the attempt and open an browser to that attempt"
        )
        .short_flag('B'),
        command_about!(
            "check_todo",
            "checks if there any TODO's in your files before uploading"
        )
        .short_flag('T'),
    ]
}


pub fn toggle(settings: Settings, arg: String) {
    let arg = arg.as_str();

    let args = vec![
        "move_node_directories",
        "upload_open_browser", 
        "check_todo"];

    if args.contains(&arg) {
        return toggle_setup(settings, arg)
    }

    eprintln!("invalid subcommand {}", arg);
}

fn toggle_setup(mut settings: Settings, key: &str) {
    let value = settings.get_bool("setup", key, false);
    let new_value = !value;
    settings.set("setup".to_string(), key.to_string(), new_value.to_string());
    println!("Updated setting\n{}: {} -> {}", key, value, new_value)
}
