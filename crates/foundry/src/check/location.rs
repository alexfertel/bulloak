//! Location utilities.
use std::fmt;

type Filename = String;
type Line = usize;

/// A code location.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Location {
    /// A code location inside a file.
    Code(Filename, Line),
    /// A file name.
    File(Filename),
}

impl Location {
    /// Returns the filename of this code location.
    pub fn file(&self) -> String {
        match self {
            Location::Code(file, _) | Location::File(file) => file.clone(),
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Location::Code(filename, line) => write!(f, "{filename}:{line}"),
            Location::File(name) => write!(f, "{name}"),
        }
    }
}
