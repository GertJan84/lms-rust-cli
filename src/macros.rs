// Macros to shorten this extreme verbose variant of subcommand matching

// ("open", _) => arguments::execute("open", "".to_string()),
// ("download", arg) => { arguments::execute("download", arg.get_one::<String>("id").unwrap().to_string()) }

// command must have the same name as the argument
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
    ($sub_cmd:ident $(,)*) => {};

    ($sub_cmd:ident, ($cmd:ident, $arg:expr) $(, $sub:tt)*) => {
        sub_inner!($sub_cmd, $cmd, $arg);
        subcommands!($sub_cmd $(, $sub)*);
    };

    ($sub_cmd:ident, $cmd:ident $(, $sub:tt)*) => {
        sub_inner!($sub_cmd, $cmd, "".to_string());
        subcommands!($sub_cmd $(, $sub)*);
    };
}

#[macro_export]
macro_rules! ustring {
    ($arg:expr) => {
        $arg.unwrap().to_string()
    };
}

#[macro_export]
macro_rules! ustr_ustring {
    ($arg:expr) => {
        ustring!($arg.unwrap().to_str())
    };
}

#[macro_export]
macro_rules! stru {
    ($arg:expr) => {
        $arg.to_str().unwrap()
    };
}

#[macro_export]
macro_rules! error_exit {
    ($($arg:tt)*) => {
        {
            eprintln!($($arg)*);
            exit(1);
        }
    };
}
