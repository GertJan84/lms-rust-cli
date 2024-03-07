use std::env;
use clap::{Command, Arg};

mod settings;
mod arguments;
mod utils;

#[macro_use]
extern crate lazy_static;


pub const CLI_VERSION: &'static str = "14";

lazy_static! {
    pub static ref BASE_URL: String = env::var("LMS_BASE_URL").unwrap_or("https://sd42.nl".to_string());    
}


fn main() {
    let cmd = Command::new("lms")
        .bin_name("lms")
        .about("Lms cli interface")
        .version(CLI_VERSION)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("login")
                .about("Connect to your sd42.nl account")
            )
        .subcommand(
            Command::new("install")
                .about("Install or upgrade lms")
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
                .about("Verify the integeity of your lms directory")
            )
        .subcommand(
            Command::new("template")
                .about("Download the current assignment template")
            )
        .subcommand(
            Command::new("download")
                .about("Download your last-submitted work for the current assignment")
                .arg(
                    Arg::new("id")
                        .help("The node id optionall followed by a '~' and a attempt number")
                        .num_args(1)
                        .required(true)
                )
            )
        .subcommand(
            Command::new("grade")
                .about("Teachers only: download everything needed for grading")
                .arg(
                    Arg::new("short_name")
                        .help("The student's short name optionall followed by '@' the node id and '~' attempt number ")
                        .num_args(1)
                        .required(true)
                )
            )
        .get_matches();

    match cmd.subcommand() {
        Some(subcommand) => {
            match subcommand {
                ("open", _) => arguments::execute("open", "".to_string()),
                ("install", _) => arguments::execute("install", "".to_string()),
                ("upload", _) => arguments::execute("upload", "".to_string()),
                ("verify", _) => arguments::execute("verify", "".to_string()),
                ("template", _) => arguments::execute("template", "".to_string()),
                ("download", arg) => arguments::execute("download", arg.get_one::<String>("id").unwrap().to_string()),
                ("grade", arg) => arguments::execute("grade", arg.get_one::<String>("short_name").unwrap().to_string()),
                _ => eprintln!("invalid command")
            }
        },
        _ => eprintln!("error")
    }
}
