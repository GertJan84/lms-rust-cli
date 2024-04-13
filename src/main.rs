extern crate glob;

use clap::{Arg, Command};
use once_cell::sync::Lazy;
use std::env;

use crate::arguments::{show::show_commands, toggle::toggle_commands};

mod arguments;
mod attempt;
mod files;
mod io;
mod prompt;
mod settings;

#[macro_use]
mod macros;

pub const CLI_VERSION: &'static str = env!("CARGO_PKG_VERSION_MAJOR");

pub static BASE_URL: Lazy<String> =
    Lazy::new(|| env::var("LMS_BASE_URL").unwrap_or("https://sd42.nl".to_string()));

fn main() {
    let mut cmd = Command::new("lms")
        .bin_name("lms")
        .about("LMS Command Line Interface")
        .version(env!("CARGO_PKG_VERSION"))
        .arg_required_else_help(true)
        .subcommand_required(true);

    // Subcommands explanations can be found by the subcommand macro
    sub_command!(
        cmd,
        "grade",
        "Teachers only: download everything needed for grading",
        "short_name",
        "The student's short name optional followed by '@' the node id and '~' attempt number",
        1,
        true
    );
    sub_command!(
        cmd,
        "download", 
        "Download submitted attempts or all attempts", 
        "id", 
        "The node id optional followed by a '~' and a attempt number or 'all' to download all attempts",
        1,
        true
    );
    sub_command!(cmd, "open", "Open the current assignment in the IDE");
    sub_command!(cmd, "login", "Connect to your sd42.nl account");
    sub_command!(cmd, "update", "Upgrade lms");
    sub_command!(cmd, "upload", "Upload your work for the current assignment");
    sub_command!(cmd, "verify", "Verify the integrity of your lms directory");
    sub_command!(cmd, "template", "Download the current assignment template");
    sub_command!(cmd, "show", "Show info from the client", show_commands());
    sub_command!(cmd, "toggle", "Toggle boolean settings", toggle_commands());

    match cmd.get_matches().subcommand() {
        Some(subcommand) => match subcommand {
            ("open", _)
            | ("login", _)
            | ("update", _)
            | ("upload", _)
            | ("verify", _)
            | ("template", _) => arguments::execute(subcommand.0, "".to_string()),
            ("download", arg) => {
                arguments::execute("download", arg.get_one::<String>("id").unwrap().to_string())
            }
            ("grade", arg) => arguments::execute(
                "grade",
                arg.get_one::<String>("short_name").unwrap().to_string(),
            ),
            ("show", sub_command) => {
                arguments::execute("show", sub_command.subcommand_name().unwrap().to_string())
            }
            ("toggle", sub_command) => {
                arguments::execute("toggle", sub_command.subcommand_name().unwrap().to_string())
            }
            _ => eprintln!("Invalid command"),
        },
        _ => eprintln!("Error"),
    }
}
