use arguments::Commands;
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

    if let Some(subcommand) = cmd.subcommand_name() {
        // TODO: Fix this
        let command = subcommand.to_string();
        let matches = Commands::get_command(command);
        //let arg = match cmd.subcommand() {
        //    Some((command, arg)) => arg,
        //    _ => unreachable!("what is happening")
        //};
        //match arg.get_one::<String>("id") {
        //    Some(return_arg) => {
        //        Commands::execute(&matches, return_arg.to_string())
        //    }, 
        //    None => {
        //        Commands::execute(&matches, "".to_string())
        //    }
        //}

        Commands::execute(&matches, "".to_string())
    }
}
