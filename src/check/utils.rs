/// Converts the start offset of a `Loc` to `(line, col)`. Modified from <https://github.com/foundry-rs/foundry/blob/45b9dccdc8584fb5fbf55eb190a880d4e3b0753f/fmt/src/helpers.rs#L54-L70>
pub(crate) fn offset_to_line_column(content: &str, start: usize) -> (usize, usize) {
    debug_assert!(content.len() > start);

    // first line is `1`
    let mut line_counter = 1;
    for (offset, c) in content.chars().enumerate() {
        if c == '\n' {
            line_counter += 1;
        }
        if offset > start {
            return (line_counter, offset - start);
        }
    }

    unreachable!("content.len() > start")
}
