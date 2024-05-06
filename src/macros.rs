#[macro_export]
macro_rules! sub_inner {
    ($sub_cmd:ident, $cmd:ident, $arg:expr) => {
        if $sub_cmd == stringify!($cmd) {
            arguments::execute(stringify!($cmd), $arg);
            return;
        }
    };
}

#[macro_export]
macro_rules! subcommands {
    ($sub_cmd:ident, $cmd:ident) => {
        sub_inner!($sub_cmd, $cmd, "".to_string());
        eprintln!("Invalid command");
    };

    ($sub_cmd:ident, ($cmd:ident, $arg:expr)) => {
        sub_inner!($sub_cmd, $cmd, $arg);
        eprintln!("Invalid command");
    };

    ($sub_cmd:ident, ($cmd:ident, $arg:expr) $(, $sub:tt)*) => {
        sub_inner!($sub_cmd, $cmd, $arg);
        subcommands!($sub_cmd, $($sub),*);
    };

    ($sub_cmd:ident, $cmd:ident $(, $sub:tt)*) => {
        sub_inner!($sub_cmd, $cmd, "".to_string());
        subcommands!($sub_cmd, $($sub),*);
    };
}
