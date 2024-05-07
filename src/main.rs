extern crate glob;

use arguments::{show::show_commands, toggle::toggle_commands};
use clap::{Arg, Command};
use once_cell::sync::Lazy;
use std::env;
mod arguments;
mod attempt;
mod files;
mod io;
mod macros;
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
        .subcommand(Command::new("review")
            .about("Send code to ai to review")
            )
        .get_matches();

    match cmd.subcommand() {
        None => eprintln!("Error"),
        Some((sub_cmd, arg)) => {
            // sub_cmd is the name of the subcommand we need to give to the macro to compare with the commands
            subcommands!(
                sub_cmd,
                login,
                update,
                upload,
                open,
                verify,
                template,
                (download, ustring!(arg.get_one::<String>("id"))),
                (show, ustring!(arg.subcommand_name())),
                (toggle, ustring!(arg.subcommand_name())),
                (grade, ustring!(arg.get_one::<String>("short_name")))
            );
            eprintln!("Invalid command");
        }
    }
}
