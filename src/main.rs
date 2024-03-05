use arguments::Commands;
use std::env;
use clap::{Command, Arg};

mod settings;
mod arguments;
mod utils;

#[macro_use]
extern crate lazy_static;


pub const CLI_VERSION: &'static str = "1";

lazy_static! {
    pub static ref BASE_URL: String = env::var("LMS_BASE_URL").unwrap_or("https://sd42.nl".to_string());    
}


fn main() {
    let cmd = Command::new("lms")
        .bin_name("lms")
        .about("Lms cli interface")
        .version("1")
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
                    Arg::new("Node id")
                        .help("the node id optionall followed by a '~' and a attempt number")
                        .num_args(1)
                )
            )
        .subcommand(
            Command::new("grade")
                .about("Teachers only: download everything needed for grading")
                .arg(
                    Arg::new("student's short name")
                        .help("The student's short name optionall followed by '@' the node id and '~' attempt number ")
                        .num_args(1)
                        .required(true)
                )
            )
        .get_matches();

    if let Some(subcommand) = cmd.subcommand_name() {
        let command = subcommand.to_string();
        let matches = Commands::get_command(command);
        Commands::execute(&matches)
    }


}
