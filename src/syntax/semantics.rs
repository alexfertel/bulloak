use std::{fmt, result};

use super::{
    ast::{self, Ast},
    span::Span,
    visitor::Visitor,
};

type Result<T> = result::Result<T, Vec<Error>>;

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

    #[cfg(test)]
    pub fn new(kind: ErrorKind, text: String, span: Span) -> Self {
        Error { kind, text, span }
    }
}

/// The type of an error that occurred while building an AST.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// Found an unexpected node. This is most probably a bug in the
    /// parser implementation.
    NodeUnexpected,
    /// Found no rules to emit.
    TreeEmpty,
    /// Found a condition with no children.
    ConditionEmpty,
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::syntax::error::Formatter::from(self).fmt(f)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ErrorKind::*;
        match *self {
            NodeUnexpected => write!(f, "unexpected child node"),
            TreeEmpty => write!(f, "no rules where defined"),
            ConditionEmpty => write!(f, "found a condition with no children"),
            _ => unreachable!(),
        }
    }
}

/// A visitor that performs semantic analysis on an AST.
pub struct SemanticAnalyzer<'t> {
    /// A list of errors that occurred while analyzing the AST.
    errors: Vec<Error>,
    /// The original text that the visitor generated the errors from. Every
    /// span in an error is a valid range into this string.
    text: &'t str,
}

impl<'t> SemanticAnalyzer<'t> {
    /// Create a new semantic analyzer.
    pub fn new(text: &'t str) -> SemanticAnalyzer {
        SemanticAnalyzer {
            errors: Vec::new(),
            text,
        }
    }

    /// Create a new error given an AST node and error type.
    fn error(&mut self, span: Span, kind: ErrorKind) {
        self.errors.push(Error {
            kind,
            text: self.text.to_string(),
            span,
        });
    }

    /// Traverse the given AST and store any errors that occur.
    ///
    /// Note that this implementation is a bit weird in that we
    /// create the `Err` variant of the result by hand.
    pub fn analyze(&mut self, ast: &ast::Ast) -> Result<()> {
        match ast {
            Ast::Root(root) => self.visit_root(root),
            Ast::Condition(condition) => self.visit_condition(condition),
            Ast::Action(action) => self.visit_action(action),
        }
        // It is fine to unwrap here since analysis errors will
        // be stored in `self.errors`.
        .unwrap();

        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }

        Ok(())
    }
}

/// A visitor that performs semantic analysis on an AST.
impl Visitor for SemanticAnalyzer<'_> {
    type Output = ();
    type Error = ();

    fn visit_root(&mut self, root: &ast::Root) -> result::Result<Self::Output, Self::Error> {
        if root.asts.is_empty() {
            self.error(Span::splat(root.span.end), ErrorKind::TreeEmpty);
        }

        for ast in &root.asts {
            match ast {
                Ast::Condition(condition) => {
                    self.visit_condition(condition)?;
                }
                Ast::Action(action) => {
                    self.visit_action(action)?;
                }
                Ast::Root(root) => {
                    self.error(root.span, ErrorKind::NodeUnexpected);
                }
            }
        }

        Ok(())
    }

    fn visit_condition(
        &mut self,
        condition: &ast::Condition,
    ) -> result::Result<Self::Output, Self::Error> {
        if condition.asts.is_empty() {
            self.error(condition.span, ErrorKind::ConditionEmpty);
        }

        for ast in &condition.asts {
            match ast {
                Ast::Condition(condition) => {
                    self.visit_condition(condition)?;
                }
                Ast::Action(action) => {
                    self.visit_action(action)?;
                }
                Ast::Root(root) => {
                    self.error(root.span, ErrorKind::NodeUnexpected);
                }
            }
        }

        Ok(())
    }

    fn visit_action(&mut self, _action: &ast::Action) -> result::Result<Self::Output, Self::Error> {
        // We don't implement any semantic rules here for now.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::syntax::ast;
    use crate::syntax::parser::Parser;
    use crate::syntax::semantics::{self, ErrorKind::*};
    use crate::syntax::span::{Position, Span};
    use crate::syntax::tokenizer::Tokenizer;

    fn analyze(text: &str) -> semantics::Result<()> {
        let tokens = Tokenizer::new().tokenize(text).unwrap();
        let ast = Parser::new().parse(text, &tokens).unwrap();
        let mut analyzer = semantics::SemanticAnalyzer::new(&text);
        analyzer.analyze(&ast)?;

        Ok(())
    }

    #[test]
    fn test_unexpected_node() {
        let ast = ast::Ast::Root(ast::Root {
            contract_name: "Foo_Test".to_string(),
            asts: vec![ast::Ast::Root(ast::Root {
                contract_name: "Foo_Test".to_string(),
                asts: vec![],
                span: Span::new(Position::new(0, 1, 1), Position::new(7, 1, 8)),
            })],
            span: Span::new(Position::new(0, 1, 1), Position::new(7, 1, 8)),
        });

        let mut analyzer = semantics::SemanticAnalyzer::new("Foo_Test");
        let result = analyzer.analyze(&ast);
        assert_eq!(
            result.unwrap_err(),
            vec![semantics::Error {
                kind: NodeUnexpected,
                text: "Foo_Test".to_string(),
                span: Span::new(Position::new(0, 1, 1), Position::new(7, 1, 8)),
            }]
        );
    }

    #[test]
    fn test_condition_empty() {
        assert_eq!(
            analyze("Foo_Test\n└── when something").unwrap_err(),
            vec![semantics::Error {
                kind: ConditionEmpty,
                text: "Foo_Test\n└── when something".to_string(),
                span: Span::new(Position::new(9, 2, 1), Position::new(32, 2, 18)),
            }]
        );
    }

    #[test]
    fn test_allow_action_without_conditions() {
        assert!(analyze("Foo_Test\n└── it a something").is_ok());
    }
}
