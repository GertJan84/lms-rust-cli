//! Fork of `textwrap` crate
//!
//! Benefits of forking:
//! - Pull in only what we need rather than relying on the compiler to remove what we don't need
//! - `LineWrapper` is able to incrementally wrap which will help with `StyledStr

pub mod core;
#[cfg(feature = "wrap_help")]
pub mod word_separators;
#[cfg(feature = "wrap_help")]
pub mod wrap_algorithms;

#[cfg(feature = "wrap_help")]
pub fn wrap(content: &str, hard_width: usize) -> String {
    let mut wrapper = wrap_algorithms::LineWrapper::new(hard_width);
    let mut total = Vec::new();
    for line in content.split_inclusive('\n') {
        wrapper.reset();
        let line = word_separators::find_words_ascii_space(line).collect::<Vec<_>>();
        total.extend(wrapper.wrap(line));
    }
    total.join("")
}

#[cfg(not(feature = "wrap_help"))]
pub fn wrap(content: &str, _hard_width: usize) -> String {
    content.to_owned()
}
