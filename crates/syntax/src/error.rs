use std::{cmp, fmt};

use crate::{span::Span, utils::repeat_str};

/// A trait for representing frontend errors in the `bulloak-syntax` crate.
///
/// This trait is implemented by various error types in the crate to provide
/// a consistent interface for error handling and formatting.
pub trait FrontendError<K: fmt::Display>: std::error::Error {
    /// Return the type of this error.
    #[must_use]
    fn kind(&self) -> &K;

    /// The original text string in which this error occurred.
    #[must_use]
    fn text(&self) -> &str;

    /// Return the span at which this error occurred.
    #[must_use]
    fn span(&self) -> &Span;

    /// Formats the error message with additional context.
    ///
    /// This method provides a default implementation that creates a formatted
    /// error message including the error kind, the relevant text, and visual
    /// indicators of where the error occurred.
    ///
    /// # Arguments
    /// * `f` - A mutable reference to a `fmt::Formatter`.
    ///
    /// # Returns
    /// A `fmt::Result` indicating whether the formatting was successful.
    fn format_error(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let divider = repeat_str("•", 79);
        writeln!(f, "{divider}")?;

        let start_offset = self.span().start.offset;
        let end_offset = self.span().end.offset;
        if start_offset == end_offset && start_offset == 0 {
            write!(f, "bulloak error: {}", self.kind())?;
            return Ok(());
        }

        writeln!(f, "bulloak error: {}\n", self.kind())?;
        let notated = self.notate();
        writeln!(f, "{notated}")?;
        writeln!(
            f,
            "--- (line {}, column {}) ---",
            self.span().start.line,
            self.span().start.column
        )?;
        Ok(())
    }

    /// Creates a string with carets (^) pointing at the span where the error
    /// occurred.
    ///
    /// This method provides a visual representation of where in the text the
    /// error was found.
    ///
    /// # Returns
    /// A `String` containing the relevant line of text with carets underneath.
    fn notate(&self) -> String {
        let mut notated = String::new();
        if let Some(line) = self.text().lines().nth(self.span().start.line - 1)
        {
            notated.push_str(line);
            notated.push('\n');
            notated.push_str(&repeat_str(" ", self.span().start.column - 1));
            let note_len =
                self.span().end.column.saturating_sub(self.span().start.column)
                    + 1;
            let note_len = cmp::max(1, note_len);
            notated.push_str(&repeat_str("^", note_len));
            notated.push('\n');
        }

        notated
    }
}

#[cfg(test)]
mod test {
    use std::fmt;

    use pretty_assertions::assert_eq;
    use thiserror::Error;

    use super::{repeat_str, FrontendError};
    use crate::span::{Position, Span};

    #[derive(Error, Clone, Debug, Eq, PartialEq)]
    pub struct Error {
        #[source]
        kind: ErrorKind,
        text: String,
        span: Span,
    }

    #[derive(Error, Clone, Debug, Eq, PartialEq)]
    #[non_exhaustive]
    pub enum ErrorKind {
        #[error("unexpected token '{0}'")]
        TokenUnexpected(String),
    }

    impl FrontendError<ErrorKind> for Error {
        fn kind(&self) -> &ErrorKind {
            &self.kind
        }

        fn text(&self) -> &str {
            &self.text
        }

        fn span(&self) -> &Span {
            &self.span
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.format_error(f)
        }
    }

    #[test]
    fn test_notate() {
        let err = Error {
            kind: ErrorKind::TokenUnexpected("world".to_owned()),
            text: "hello\nworld\n".to_owned(),
            span: Span::new(Position::new(0, 2, 1), Position::new(4, 2, 5)),
        };
        let notated = format!("{}", err);

        let mut expected = String::from("");
        expected.push_str(&repeat_str("•", 79));
        expected.push('\n');
        expected
            .push_str(format!("bulloak error: {}\n\n", err.kind()).as_str());
        expected.push_str("world\n");
        expected.push_str("^^^^^\n\n");
        expected.push_str(
            format!(
                "--- (line {}, column {}) ---\n",
                err.span().start.line,
                err.span().start.column
            )
            .as_str(),
        );
        assert_eq!(expected, notated);
    }
}
