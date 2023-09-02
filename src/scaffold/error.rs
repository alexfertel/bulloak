use std::cmp;
use std::fmt;
use std::result;

use super::parser;
use super::semantics;
use super::span;
use super::tokenizer;

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
    Semantic(Vec<semantics::Error>),
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

impl From<Vec<semantics::Error>> for Error {
    fn from(errors: Vec<semantics::Error>) -> Error {
        Error::Semantic(errors)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Parse(ref x) => x.fmt(f),
            Error::Tokenize(ref x) => x.fmt(f),
            Error::Semantic(ref errors) => {
                for x in errors {
                    x.fmt(f)?;
                }
                Ok(())
            }
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
        writeln!(f, "bulloak error: {}\n", self.err)?;
        let notated = notate(self);
        writeln!(f, "{}", notated)?;
        writeln!(
            f,
            "--- (line {}, column {}) ---",
            self.span.start.line, self.span.start.column
        )?;
        Ok(())
    }
}

/// Notate the text string with carets (`^`) pointing at the span.
fn notate<E>(f: &Formatter<'_, E>) -> String {
    let mut notated = String::new();
    if let Some(line) = f.text.lines().nth(f.span.start.line - 1) {
        notated.push_str(line);
        notated.push('\n');
        notated.push_str(&repeat_str(" ", f.span.start.column - 1));
        let note_len = f.span.end.column.saturating_sub(f.span.start.column) + 1;
        let note_len = cmp::max(1, note_len);
        notated.push_str(&repeat_str("^", note_len));
        notated.push('\n');
    }

    notated
}

fn repeat_str(s: &str, n: usize) -> String {
    s.repeat(n)
}

#[cfg(test)]
mod test {
    use super::repeat_str;
    use crate::scaffold::error::Formatter;
    use crate::scaffold::span::{Position, Span};
    use crate::scaffold::{error, parser, semantics};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_notate() {
        let text = "hello\nworld\n";
        let span = Span::new(Position::new(0, 2, 1), Position::new(4, 2, 5));
        let formatter = Formatter {
            text,
            err: &parser::ErrorKind::TokenUnexpected("world".to_string()),
            span: &span,
        };
        let notated = format!("{}", formatter);

        let mut expected = String::from("");
        expected.push_str(&repeat_str("•", 79));
        expected.push('\n');
        expected.push_str(format!("bulloak error: {}\n\n", formatter.err).as_str());
        expected.push_str("world\n");
        expected.push_str("^^^^^\n\n");
        expected.push_str(
            format!(
                "--- (line {}, column {}) ---\n",
                formatter.span.start.line, formatter.span.start.column
            )
            .as_str(),
        );
        assert_eq!(expected, notated);
    }

    #[test]
    fn test_multiple_errors() {
        let text = r"test.sol
├── when 1
└── when 2"
            .to_string();

        let errors = error::Error::from(vec![
            semantics::Error::new(
                semantics::ErrorKind::ConditionEmpty,
                text.clone(),
                Span::new(Position::new(9, 2, 1), Position::new(18, 2, 10)),
            ),
            semantics::Error::new(
                semantics::ErrorKind::ConditionEmpty,
                text.clone(),
                Span::new(Position::new(20, 3, 1), Position::new(29, 3, 10)),
            ),
        ]);
        let actual = format!("{errors}");

        let expected = r"•••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••
bulloak error: found a condition with no children

├── when 1
^^^^^^^^^^

--- (line 2, column 1) ---
•••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••
bulloak error: found a condition with no children

└── when 2
^^^^^^^^^^

--- (line 3, column 1) ---
";

        assert_eq!(expected, actual);
    }
}
