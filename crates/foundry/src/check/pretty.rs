//! Pretty‑print utilities for Solang diagnostics.
//!
//! This module provides the [`Pretty`] wrapper which formats
//! [`solang_parser::diagnostics::Diagnostic`] values in a compact,
//! human‑readable style that resembles `rustc` output:
//!
//! ```text
//! path/to/file.sol:12:34 error [syntax] unexpected token
//!   note: a more detailed explanation of the problem
//! ```
//!
//! The main entry point is [`Pretty::new`]; create a value and use the
//! [`Display`] implementation to render the diagnostic.
//!
//! Helper functionality includes [`ErrorTypeExt::as_str`], converting Solang's
//! [`ErrorType`] enumeration to short lowercase strings, and
//! [`byte_to_line_col`], which translates a byte index into 1‑based line and
//! column numbers.
//!
//! # Example
//!
//! ```rust,ignore
//! let pretty = Pretty::new(&diag, "contract.sol", &source);
//! println!("{}", pretty); // prints a single‑line diagnostic
//! ```

use std::fmt::{self, Display, Formatter};

use solang_parser::{
    diagnostics::{Diagnostic, ErrorType},
    pt::Loc,
};

/// Convenience extension trait for converting [`ErrorType`] values to strings.
pub trait ErrorTypeExt {
    /// Returns a short lowercase string such as `"syntax"` or `"type"` that
    /// represents the variant, or an empty string when the variant is
    /// [`ErrorType::None`].
    fn as_str(&self) -> &'static str;
}

impl ErrorTypeExt for solang_parser::diagnostics::ErrorType {
    #[rustfmt::skip]
    fn as_str(&self) -> &'static str {
        use solang_parser::diagnostics::ErrorType::*;
        match self {
            None             => "none",
            ParserError      => "parser",
            SyntaxError      => "syntax",
            DeclarationError => "declaration",
            CastError        => "cast",
            TypeError        => "type",
            Warning          => "warning",
        }
    }
}

/// A pretty‑printer for [`Diagnostic`] values.
///
/// `Pretty` keeps references to the diagnostic itself, the filename, and the
/// full source code so that it can map byte offsets to line/column numbers.
#[derive(Clone, Copy)]
pub struct Pretty<'a> {
    diagnostic: &'a Diagnostic,
    filename: &'a str,
    source: &'a str,
}

impl<'a> Pretty<'a> {
    /// Create a new [`Pretty`] wrapper.
    ///
    /// * `diagnostic` – the diagnostic to pretty‑print.
    /// * `filename` – the file name to display in the output.
    /// * `source` – the full source text corresponding to `filename`.
    pub fn new(
        diagnostic: &'a Diagnostic,
        filename: &'a str,
        source: &'a str,
    ) -> Self {
        Pretty { diagnostic, filename, source }
    }
}

impl Pretty<'_> {
    /// Render the location part (`<file>:<line>:<col>` or a special marker) of
    /// the diagnostic.
    pub fn fmt_loc(&self) -> String {
        match self.diagnostic.loc {
            Loc::Builtin => "<builtin>".to_string(),
            Loc::CommandLine => "<cmdline>".to_string(),
            Loc::Implicit => "<implicit>".to_string(),
            Loc::Codegen => "<codegen>".to_string(),
            Loc::File(_, byte, _) => {
                let (l, c) = byte_to_line_col(self.source, byte);
                format!("{}:{l}:{c}", self.filename)
            }
        }
    }
}

impl<'d> Display for Pretty<'d> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let diag = self.diagnostic;

        // ──  file:line:col  error|warning  [ErrorType]  message
        let loc_str = self.fmt_loc();
        if let ErrorType::None = diag.ty {
            writeln!(f, "{}: {} {}", loc_str, diag.level, diag.message)?;
        } else {
            writeln!(
                f,
                "{}: {} [{}] {}",
                loc_str,
                diag.level,
                diag.ty.as_str(),
                diag.message
            )?;
        }

        // Each note on its own indented line.
        for note in &diag.notes {
            writeln!(f, "  note: {}", note.message)?;
        }
        Ok(())
    }
}

/// Convert a byte offset within `src` to 1‑based (line, column).
///
/// Traverses `src` up to the offset and counts newlines to determine the line
/// number, resetting the column counter after each `\n`.
fn byte_to_line_col(src: &str, byte: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;

    for b in src[..byte.min(src.len())].bytes() {
        if b == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use solang_parser::diagnostics::{Diagnostic, Level, Note};

    use super::*;

    /// A helper that builds a minimal `Diagnostic` we can wrap in `Pretty`.
    fn make_diag(
        loc: Loc,
        level: Level,
        ty: ErrorType,
        msg: &str,
        notes: Vec<Note>,
    ) -> Diagnostic {
        Diagnostic { loc, level, ty, message: msg.to_owned(), notes }
    }

    #[test]
    fn fmt_loc_translates_byte_offsets() {
        let source = "line1\nline2\nline3";
        // byte index  012345 6 789012 3 45678
        // byte 13 is 'i' in "line3"  ->  line 3, col 2
        let loc = Loc::File(0, 13, 0);
        let diag = make_diag(loc, Level::Info, ErrorType::None, "msg", vec![]);

        let pretty = Pretty { diagnostic: &diag, filename: "test.sol", source };
        assert_eq!(pretty.fmt_loc(), "test.sol:3:2");
    }

    #[test]
    fn fmt_loc_handles_special_locations() {
        let diag = make_diag(
            Loc::Builtin,
            Level::Debug,
            ErrorType::None,
            "msg",
            vec![],
        );
        let pretty =
            Pretty { diagnostic: &diag, filename: "ignored.sol", source: "" };
        assert_eq!(pretty.fmt_loc(), "<builtin>");
    }

    #[test]
    fn display_formats_with_and_without_error_type() {
        // 1. With an explicit ErrorType
        let source = "x = 1;";
        let err_loc = Loc::File(0, 0, 0);
        let diag1 = make_diag(
            err_loc,
            Level::Error,
            ErrorType::SyntaxError,
            "unexpected token",
            vec![],
        );
        let pretty1 =
            Pretty { diagnostic: &diag1, filename: "code.sol", source };
        let rendered1 = pretty1.to_string();
        assert!(
            rendered1
                .starts_with("code.sol:1:1: error [syntax] unexpected token"),
            "rendered: {rendered1:?}"
        );

        // 2. Without an ErrorType (ErrorType::None)
        let diag2 = make_diag(
            err_loc,
            Level::Warning,
            ErrorType::None,
            "unused variable",
            vec![],
        );
        let pretty2 =
            Pretty { diagnostic: &diag2, filename: "code.sol", source };
        let rendered2 = pretty2.to_string();
        assert!(
            rendered2.starts_with("code.sol:1:1: warning unused variable"),
            "rendered: {rendered2:?}"
        );
    }
}
