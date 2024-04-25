extern crate glob;

use arguments::{show::show_commands, toggle::toggle_commands};
use clap::{Arg, Command};
use once_cell::sync::Lazy;
use std::env;
mod arguments;
mod attempt;
mod files;
mod io;
mod prompt;
mod settings;
mod tests;

pub const CLI_VERSION: &'static str = env!("CARGO_PKG_VERSION_MAJOR");

pub static BASE_URL: Lazy<String> =
    Lazy::new(|| env::var("LMS_BASE_URL").unwrap_or("https://sd42.nl".to_string()));

fn main() {
    let cmd = Command::new("lms")
        .bin_name("lms")
        .about("LMS Command Line Interface")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("login")
                .about("Connect to your sd42.nl account")
            )
        .subcommand(
            Command::new("update")
                .about("Upgrade lms")
            )
        .subcommand(
            Command::new("upload")
                .about("Upload your work for the current assignment")
            )
        .subcommand(
            Command::new("open")
                .about("Open the current assignment in the IDE")
            )
        .subcommand(
            Command::new("verify")
                .about("Verify the integrity of your lms directory")
            )
        .subcommand(
            Command::new("template")
                .about("Download the current assignment template")
            )
        .subcommand(
            Command::new("download")
                .about("Download submitted attempts or all attempts")
                .arg(
                    Arg::new("id")
                        .help("The node id optional followed by a '~' and a attempt number or 'all' to download all attempts")
                        .num_args(1)
                        .required(true)
                )
            )
        .subcommand(
            Command::new("grade")
                .about("Teachers only: download everything needed for grading")
                .arg(
                    Arg::new("short_name")
                        .help("The student's short name optional followed by '@' the node id and '~' attempt number ")
                        .num_args(1)
                        .required(true)
                )
            )
        .subcommand(Command::new("show")
            .subcommands(show_commands())
            .about("Show info from the client")
            .arg_required_else_help(true)
            )
        .subcommand(Command::new("toggle")
            .subcommands(toggle_commands())
            .about("Toggle settings true or false")
            .arg_required_else_help(true)
            )

        .get_matches();

    match cmd.subcommand() {
        Some(subcommand) => match subcommand {
            ("open", _) => arguments::execute("open", "".to_string()),
            ("login", _) => arguments::execute("login", "".to_string()),
            ("update", _) => arguments::execute("update", "".to_string()),
            ("upload", _) => arguments::execute("upload", "".to_string()),
            ("verify", _) => arguments::execute("verify", "".to_string()),
            ("template", _) => arguments::execute("template", "".to_string()),
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
