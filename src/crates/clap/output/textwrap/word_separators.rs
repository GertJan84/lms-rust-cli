pub fn find_words_ascii_space(line: &str) -> impl Iterator<Item = &'_ str> + '_ {
    let mut start = 0;
    let mut in_whitespace = false;
    let mut char_indices = line.char_indices();

    std::iter::from_fn(move || {
        for (idx, ch) in char_indices.by_ref() {
            let next_whitespace = ch == ' ';
            if in_whitespace && !next_whitespace {
                let word = &line[start..idx];
                start = idx;
                in_whitespace = next_whitespace;
                return Some(word);
            }

            in_whitespace = next_whitespace;
        }

        if start < line.len() {
            let word = &line[start..];
            start = line.len();
            return Some(word);
        }

        None
    })
}
