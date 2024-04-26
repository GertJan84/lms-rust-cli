mod help;
#[cfg(feature = "help")]
mod help_template;
mod usage;

pub mod fmt;
#[cfg(feature = "help")]
pub mod textwrap;

pub use self::help::write_help;
#[cfg(feature = "help")]
pub use self::help_template::AutoHelp;
#[cfg(feature = "help")]
pub use self::help_template::HelpTemplate;
#[cfg(feature = "help")]
pub use self::textwrap::core::display_width;
#[cfg(feature = "help")]
pub use self::textwrap::wrap;
pub use self::usage::Usage;

pub const TAB: &str = "  ";
#[cfg(feature = "help")]
pub const TAB_WIDTH: usize = TAB.len();
