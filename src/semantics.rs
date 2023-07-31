use std::{fmt, result};

use crate::{
    ast::{self, Ast},
    span::Span,
    visitor::Visitor,
};

type Result<T> = result::Result<T, Error>;

/// An error that occurred while doing semantic analysis on the abstract
/// syntax tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    /// The kind of error.
    kind: ErrorKind,
    /// The original text that the visitor generated the error from. Every
    /// span in an error is a valid range into this string.
    text: String,
    /// The span of this error.
    span: Span,
}

impl Error {
    /// Return the type of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// The original text string in which this error occurred.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the span at which this error occurred.
    pub fn span(&self) -> &Span {
        &self.span
    }
}

/// The type of an error that occurred while building an AST.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// The output filename should have a .sol extension.
    InvalidFileName,
    /// Found an unexpected node. This is most probably a bug in the
    /// parser implementation.
    UnexpectedNode,
    /// Found no rules to emit.
    EmptyTree,
    /// Found an action with no conditions.
    ActionWithoutConditions,
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::error::Formatter::from(self).fmt(f)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ErrorKind::*;
        match *self {
            InvalidFileName => write!(f, "output filename should have a .sol extension"),
            UnexpectedNode => write!(f, "unexpected child node"),
            EmptyTree => write!(f, "no rules where defined"),
            ActionWithoutConditions => write!(f, "found an action without conditions"),
            _ => unreachable!(),
        }
    }
}

pub struct SemanticAnalyzer<'t> {
    pub errors: Vec<Error>,
    text: &'t str,
}

impl SemanticAnalyzer<'_> {
    pub fn new(text: &str) -> SemanticAnalyzer {
        SemanticAnalyzer {
            errors: Vec::new(),
            text,
        }
    }

    /// Create a new error given an AST node and error type.
    fn error(&self, span: Span, kind: ErrorKind) -> Error {
        Error {
            kind,
            text: self.text.to_string(),
            span,
        }
    }

    pub fn analyze(&mut self, root: &ast::Root) -> Result<&Vec<Error>> {
        let _ = self.visit_root(root);
        Ok(&self.errors)
    }
}

impl Visitor for SemanticAnalyzer<'_> {
    type Output = ();
    type Error = ();

    fn visit_root(&mut self, root: &ast::Root) -> result::Result<Self::Output, Self::Error> {
        if !is_valid_sol_filename(&root.file_name) {
            self.errors
                .push(self.error(root.span, ErrorKind::InvalidFileName));
        }

        if root.asts.is_empty() {
            self.errors
                .push(self.error(Span::splat(root.span.end), ErrorKind::EmptyTree));
        }

        root.asts.iter().for_each(|ast| match ast {
            Ast::Condition(condition) => {
                let _ = self.visit_condition(condition);
            }
            Ast::Action(action) => {
                self.errors
                    .push(self.error(action.span, ErrorKind::ActionWithoutConditions));
            }
            Ast::Root(root) => {
                self.errors
                    .push(self.error(root.span, ErrorKind::UnexpectedNode));
            }
        });

        Ok(())
    }

    fn visit_condition(
        &mut self,
        _condition: &ast::Condition,
    ) -> result::Result<Self::Output, Self::Error> {
        // We don't implement any semantic rules here for now.
        Ok(())
    }

    fn visit_action(&mut self, _action: &ast::Action) -> result::Result<Self::Output, Self::Error> {
        // We don't implement any semantic rules here for now.
        Ok(())
    }
}

fn is_valid_sol_filename(filename: &str) -> bool {
    filename.ends_with(".sol")
}
