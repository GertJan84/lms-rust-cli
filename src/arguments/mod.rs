mod download;
mod grade;
mod ide;
mod logics;
mod login;
mod setups;

use std::process::exit;

const AUTH_TOKEN_LENGTH: u8 = 69;
const SCAN_FILE_TYPE: [&str; 7] = ["sql", "rs", "py", "js", "css", "html", "svelte"];
const DOWNLOAD_EXCLUDE: [&str; 3] = ["exam", "project", "graduation"];

pub fn execute(command: &str, arg: String) {
    let settings = crate::settings::Settings::new();
    match command {
        "open" => logics::open_logic(&settings),
        "grade" => grade::grade_logic(&settings, arg),
        "upload" => logics::upload_logic(&settings),
        "download" => download::download_logic(&settings, arg),
        "template" => logics::template_logic(&settings),
        "update" => crate::io::handle_upgrade(),
        "verify" => setups::verify_logic(),
        "login" => login::login_logic(settings),
        "show" => crate::show::show(&settings, arg),
        "toggle" => crate::toggle::toggle(settings, arg),
        _ => {
            eprintln!("invalid command {}", command);
        }
    }

    exit(1)
}
