// Possible TODO: refactor???? for me it is understandable what happens.... Don't know if the rest knows it.
// Stan

#[macro_export]
macro_rules! command_about {
    // Command with about
    ($name:expr, $about:expr) => {
        Command::new($name).about($about)
    };
}

#[macro_export]
macro_rules! argument {
    // Creating an argument
    ($name:expr, $help:expr, $args_count:expr, $required:expr) => {
        Arg::new($name)
            .help($help)
            .num_args($args_count)
            .required($required)
    };
}

#[macro_export]
macro_rules! sub_command {
    // Simple subcommand with only a Command
    ($cmd:expr, $name:expr, $about:expr) => {
        $cmd = $cmd.subcommand(command_about!($name, $about))
    };
    ($cmd:expr, $name:expr, $about:expr, $subcommands:expr) => {
        // Subcommand with subcommands that always needs to be true.
        // $subcommands are need to be made like in show.rs or toggle.rs
        $cmd = $cmd.subcommand(
            command_about!($name, $about)
                .subcommands($subcommands)
                .arg_required_else_help(true),
        )
    };
    ($cmd:expr, $name:expr, $about:expr, $subcommands:expr, $required:expr) => {
        // Subcommand with subcommands that can be set true or false.
        // $subcommands are need to be made like in show.rs or toggle.rs
        $cmd = $cmd.subcommand(
            command_about!($name, $about)
                .subcommands($subcommands)
                .arg_required_else_help($required),
        )
    };
    ($cmd:expr, $name:expr, $about:expr, $arg_name:expr, $arg_help:expr, $args_count:expr, $required:expr) => {
        // Subcommand with arguments that always need to have the count and required
        // (otherwise it will have had the same amount of parameters as the other one)
        $cmd = $cmd.subcommand(command_about!($name, $about).arg(argument!(
            $arg_name,
            $arg_help,
            $args_count,
            $required
        )))
    };
}
