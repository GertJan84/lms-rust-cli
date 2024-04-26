// use clap::Command;

use crate::crates::clap::Command;
use crate::settings::Settings;

// use crate::{crates::clap::Command, settings::Settings};

pub fn toggle_commands() -> Vec<Command> {
    vec![
        Command::new("move_node_directories")
            .about("update your file structure so it matches correct")
            .short_flag('D'),
        Command::new("upload_open_browser")
            .about("upload the attempt and open an browser to that attempt")
            .short_flag('B'),
        Command::new("check_todo")
            .about("checks if there any TODO's in your files before uploading")
            .short_flag('T'),
        Command::new("open_first_folder")
            .about("opens first empty folder (opens android-studio correctly)")
            .short_flag('O'),
    ]
}

pub fn toggle(mut settings: &mut Settings, arg: String) {
    let key = arg.as_str();

    if toggle_commands()
        .iter()
        .any(|command| command.get_name() == key)
    {
        toggle_setup(&mut settings, key);
    } else {
        eprintln!("invalid subcommand {}", key);
    }
}

fn toggle_setup(settings: &mut Settings, key: &str) {
    let value = settings.get_bool("setup", key, false);
    let new_value = !value;
    settings.set("setup".to_string(), key.to_string(), new_value.to_string());
    println!("Updated setting\n{}: {} -> {}", key, value, new_value)
}

#[cfg(test)]
mod tests {
    use crate::tests::create_test_settings;

    use super::*;

    #[test]
    fn test_toggle_setup() {
        let mut settings = create_test_settings();
        toggle_setup(&mut settings, "check_todo");
        assert_eq!(settings.get_bool("setup", "check_todo", false), false);
    }
}
