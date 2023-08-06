pub fn capitalize_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// This functions makes the appropriate changes to a string to
/// make it a valid identifier.
pub fn sanitize(identifier: &str) -> String {
    identifier
        .replace("-", "_")
        .replace("'", "")
        .replace("\"", "")
}
