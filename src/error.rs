use std::cmp;
use std::fmt;
use std::result;

use crate::parser;
use crate::semantics;
use crate::span;
use crate::tokenizer;

/// A type alias for dealing with errors returned by this crate.
pub type Result<T> = result::Result<T, Error>;

/// This error type encompasses any error that can be returned by this crate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// An error that occurred while translating abstract syntax into a high
    /// level intermediate representation.
    Tokenize(tokenizer::Error),
    /// An error that occurred while translating concrete syntax into abstract
    /// syntax.
    Parse(parser::Error),
    /// An error that occurred while doing semantic analysis on the abstract
    /// syntax tree.
    Semantic(semantics::Error),
    /// Hints that destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl From<parser::Error> for Error {
    fn from(err: parser::Error) -> Error {
        Error::Parse(err)
    }
}

impl From<tokenizer::Error> for Error {
    fn from(err: tokenizer::Error) -> Error {
        Error::Tokenize(err)
    }
}

impl From<semantics::Error> for Error {
    fn from(err: semantics::Error) -> Error {
        Error::Semantic(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Parse(ref x) => x.fmt(f),
            Error::Tokenize(ref x) => x.fmt(f),
            Error::Semantic(ref x) => x.fmt(f),
            _ => unreachable!(),
        }
    }
}

/// A helper type for formatting nice error messages.
///
/// This type is responsible for reporting errors in a nice human readable
/// format.
#[derive(Debug)]
pub struct Formatter<'e, E> {
    /// The original .tree text in which the error occurred.
    text: &'e str,
    /// The error kind. It must impl fmt::Display.
    err: &'e E,
    /// The span of the error.
    span: &'e span::Span,
}

impl<'e> From<&'e parser::Error> for Formatter<'e, parser::ErrorKind> {
    fn from(err: &'e parser::Error) -> Self {
        Formatter {
            text: err.text(),
            err: err.kind(),
            span: err.span(),
        }
    }
}

impl<'e> From<&'e tokenizer::Error> for Formatter<'e, tokenizer::ErrorKind> {
    fn from(err: &'e tokenizer::Error) -> Self {
        Formatter {
            text: err.text(),
            err: err.kind(),
            span: err.span(),
        }
    }
}

impl<'e> From<&'e semantics::Error> for Formatter<'e, semantics::ErrorKind> {
    fn from(err: &'e semantics::Error) -> Self {
        Formatter {
            text: err.text(),
            err: err.kind(),
            span: err.span(),
        }
    }
}

impl<'e, E: fmt::Display> fmt::Display for Formatter<'e, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let divider = repeat_str("•", 79);
        writeln!(f, "{}", divider)?;
        writeln!(f, "bulloak error:")?;
        let notated = notate(&self);
        write!(f, "{}", notated)?;
        write!(f, "error: {}", self.err)?;
        Ok(())
    }
}

/// Notate the text string with carets (`^`) pointing at the span.
fn notate<'e, E>(f: &Formatter<'e, E>) -> String {
    let mut notated = String::new();
    if let Some(line) = f.text.lines().nth(f.span.start.line - 1) {
        notated.push_str(line);
        notated.push('\n');
        notated.push_str(&repeat_str(" ", f.span.start.column - 1));
        let note_len = f.span.end.column.saturating_sub(f.span.start.column) + 1;
        let note_len = cmp::max(1, note_len);
        notated.push_str(&repeat_str("^", note_len));
        notated.push('\n');
        notated.push('\n');
    }

    notated
}

fn repeat_str(s: &str, n: usize) -> String {
    ::std::iter::repeat(s).take(n).collect()
}

#[cfg(test)]
mod test {
    use super::{repeat_str, Formatter};
    use crate::span::{Position, Span};
    use crate::tokenizer;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_notate() {
        let text = "hello\nworld\n";
        let span = Span::splat(Position::new(0, 2, 1)).with_end(Position::new(0, 2, 5));
        let formatter = Formatter {
            text,
            err: &tokenizer::ErrorKind::InvalidCharacter('w'),
            span: &span,
        };
        let notated = format!("{}", formatter);

        let mut expected = String::from("");
        expected.push_str(&repeat_str("•", 79));
        expected.push('\n');
        expected.push_str("bulloak error:\n");
        expected.push_str("world\n");
        expected.push_str("^^^^^\n\n");
        expected.push_str(format!("error: {}", formatter.err).as_str());
        assert_eq!(notated, expected);
    }
}
