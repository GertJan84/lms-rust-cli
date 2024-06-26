pub mod download;
pub mod grade;
pub mod ide;
pub mod logics;
pub mod login;
pub mod review;
pub mod setups;
pub mod show;
pub mod toggle;

use std::process::exit;

const AUTH_TOKEN_LENGTH: u8 = 69;
const SCAN_FILE_TYPE: [&str; 7] = ["sql", "rs", "py", "js", "css", "html", "svelte"];
const DOWNLOAD_EXCLUDE: [&str; 3] = ["exam", "project", "graduation"];

pub fn execute(command: &str, arg: String) {
    let mut settings = crate::settings::Settings::new();
    match command {
        "open" => logics::open_logic(&settings),
        "grade" => grade::grade_logic(&settings, arg),
        "upload" => logics::upload_logic(&settings),
        "download" => download::download_logic(&settings, arg),
        "template" => logics::template_logic(&settings),
        "update" => crate::io::handle_upgrade(),
        "verify" => setups::verify_logic(),
        "login" => login::login_logic(settings),
        "show" => show::show(&settings, arg),
        "toggle" => toggle::toggle(&mut settings, arg),
        "review" => review::review(&settings),
        _ => eprintln!("invalid command {}", command),
    }

    exit(1)
}
