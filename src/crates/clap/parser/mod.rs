//! [`Command`][crate::Command] line argument parser

mod arg_matcher;
mod error;
mod matches;
#[allow(clippy::module_inception)]
mod parser;
mod validator;

pub mod features;

pub use self::arg_matcher::ArgMatcher;
pub use self::matches::{MatchedArg, SubCommand};
pub use self::parser::Identifier;
pub use self::parser::PendingArg;
pub use self::parser::{ParseState, Parser};
pub use self::validator::get_possible_values_cli;
pub use self::validator::Validator;

pub use self::matches::IdsRef;
pub use self::matches::RawValues;
pub use self::matches::Values;
pub use self::matches::ValuesRef;
pub use self::matches::{ArgMatches, Indices, ValueSource};
pub use error::MatchesError;
