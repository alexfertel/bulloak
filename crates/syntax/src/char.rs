pub(crate) trait CharExt {
    /// Checks whether a character can appear in an identifier.
    ///
    /// Valid identifiers are those which can be used as a variable name
    /// plus `-`, which will be converted to `_` in the generated code.
    fn is_valid_identifier(&self) -> bool;
}

impl CharExt for char {
    fn is_valid_identifier(&self) -> bool {
        self.is_alphanumeric()
            || *self == '_'
            || *self == '-'
            || *self == '\''
            || *self == '"'
    }
}
