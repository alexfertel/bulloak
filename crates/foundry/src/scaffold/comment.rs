//! Helpers for normalizing scaffolded comments.

/// Normalize a description by capitalizing its first alphabetic character and
/// ensuring it ends with a dot, while preserving surrounding whitespace.
pub(crate) fn normalize(lexeme: &str) -> String {
    let (prefix, core, _suffix) = split_whitespace_affixes(lexeme);

    if core.is_empty() {
        return lexeme.to_string();
    }

    let mut normalized = core.to_string();
    capitalize_first_alpha(&mut normalized);
    ensure_terminal_dot(&mut normalized);

    format!("{prefix}{normalized}")
}

fn capitalize_first_alpha(s: &mut String) {
    if let Some((idx, ch)) = s.char_indices().find(|(_, ch)| ch.is_alphabetic())
    {
        let uppercase = ch.to_uppercase().to_string();
        s.replace_range(idx..idx + ch.len_utf8(), &uppercase);
    }
}

fn ensure_terminal_dot(s: &mut String) {
    let trimmed_len = s.trim_end().len();
    if trimmed_len == 0 {
        return;
    }

    let trimmed = &s[..trimmed_len];
    if let Some((idx, ch)) = trimmed.char_indices().last() {
        if ch == '.' {
            return;
        }

        if matches!(ch, '!' | '?') {
            s.replace_range(idx..idx + ch.len_utf8(), ".");
        } else {
            s.insert(idx + ch.len_utf8(), '.');
        }
    }
}

fn split_whitespace_affixes(s: &str) -> (&str, &str, &str) {
    let mut prefix_len = 0usize;
    for (idx, ch) in s.char_indices() {
        if ch.is_whitespace() {
            prefix_len = idx + ch.len_utf8();
        } else {
            break;
        }
    }

    let mut suffix_len = 0usize;
    for (_, ch) in s.char_indices().rev() {
        if ch.is_whitespace() {
            suffix_len += ch.len_utf8();
        } else {
            break;
        }
    }

    let core_end = s.len().saturating_sub(suffix_len);
    let core_start = prefix_len.min(core_end);

    (&s[..core_start], &s[core_start..core_end], &s[core_end..])
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn preserves_whitespace() {
        assert_eq!(normalize("    hello"), "    Hello.");
        assert_eq!(normalize("hello   "), "Hello.");
        assert_eq!(normalize("  hello   "), "  Hello.");
    }

    #[test]
    fn capitalizes_and_dots() {
        assert_eq!(normalize("foo"), "Foo.");
        assert_eq!(normalize("Foo."), "Foo.");
        assert_eq!(normalize("foo!"), "Foo.");
        assert_eq!(normalize("foo?"), "Foo.");
        assert_eq!(normalize("FOO"), "FOO.");
    }

    #[test]
    fn handles_empty_core() {
        assert_eq!(normalize("   "), "   ");
        assert_eq!(normalize(""), "");
    }
}
