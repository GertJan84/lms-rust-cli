use std::io::Write;

pub fn yes_no(message: &str) -> bool {
    loop {
        print!("{} [Y, n]: ", message);
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to get input");

        let trim_input = input.trim().to_lowercase();

        match trim_input.as_str() {
            "y" | "" => return true,
            "n" => return false,
            _ => println!("{}: is not valid", trim_input),
        }
    }
}
