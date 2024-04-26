pub use builder::Arg;
pub use builder::Command;
pub use parser::ArgMatches;
pub use util::color::ColorChoice;
pub use util::Id;

/// Command Line Argument Parser Error
///
/// See [`Command::error`] to create an error.
///
/// [`Command::error`]: crate::Command::error
pub type Error = error::Error<error::DefaultFormatter>;

#[macro_use]
pub mod macros;

pub mod builder;
pub mod clap_lex;
pub mod derive;
pub mod error;
pub mod mkeymap;
pub mod output;
pub mod parser;
pub mod util;

const INTERNAL_ERROR_MSG: &str = "Fatal internal error. Please consider filing a bug \
                                  report at https://github.com/clap-rs/clap/issues";
