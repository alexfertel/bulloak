use std::fmt;

type Filename = String;
type Line = usize;

/// A code location.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Location {
    /// A code location inside a file.
    Code(Filename, Line),
    /// A file name.
    File(Filename),
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Location::Code(filename, line) => write!(f, "file: {filename} | line: {line}"),
            Location::File(name) => write!(f, "file: {name}"),
        }
    }
}
