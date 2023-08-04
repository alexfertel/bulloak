use std::{fmt, result};

use crate::{
    ast::{self, Ast},
    span::{Position, Span},
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
}

/// The type of an error that occurred while building an AST.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// The output filename should have a .sol extension.
    FileExtensionInvalid,
    /// Found an unexpected node. This is most probably a bug in the
    /// parser implementation.
    NodeUnexpected,
    /// Found no rules to emit.
    TreeEmpty,
    /// Found an action with no conditions.
    ActionWithoutConditions,
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
        crate::error::Formatter::from(self).fmt(f)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ErrorKind::*;
        match *self {
            FileExtensionInvalid => write!(f, "output filename should have a .sol extension"),
            NodeUnexpected => write!(f, "unexpected child node"),
            TreeEmpty => write!(f, "no rules where defined"),
            ActionWithoutConditions => write!(f, "found an action without conditions"),
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
    fn error(&self, span: Span, kind: ErrorKind) -> Error {
        Error {
            kind,
            text: self.text.to_string(),
            span,
        }
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
        if !is_valid_sol_filename(&root.file_name) {
            let span = root.span;
            let end = Position::new(
                span.start.offset + root.file_name.len() - 1,
                span.start.line,
                span.start.column + root.file_name.len() - 1,
            );
            self.errors
                .push(self.error(span.with_end(end), ErrorKind::FileExtensionInvalid));
        }

        if root.asts.is_empty() {
            self.errors
                .push(self.error(Span::splat(root.span.end), ErrorKind::TreeEmpty));
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
                    .push(self.error(root.span, ErrorKind::NodeUnexpected));
            }
        });

        Ok(())
    }

    fn visit_condition(
        &mut self,
        condition: &ast::Condition,
    ) -> result::Result<Self::Output, Self::Error> {
        if condition.asts.is_empty() {
            self.errors
                .push(self.error(condition.span, ErrorKind::ConditionEmpty));
        }

        for ast in &condition.asts {
            match ast {
                Ast::Condition(condition) => {
                    let _ = self.visit_condition(condition);
                }
                Ast::Action(action) => {
                    let _ = self.visit_action(action);
                }
                Ast::Root(root) => {
                    self.errors
                        .push(self.error(root.span, ErrorKind::NodeUnexpected));
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

fn is_valid_sol_filename(filename: &str) -> bool {
    filename.ends_with(".sol")
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::ast;
    use crate::parser::Parser;
    use crate::semantics::{self, ErrorKind::*};
    use crate::span::{Position, Span};
    use crate::tokenizer::Tokenizer;

    fn analyze(text: &str) -> semantics::Result<()> {
        let tokens = Tokenizer::new().tokenize(text).unwrap();
        let ast = Parser::new().parse(text, &tokens).unwrap();
        let mut analyzer = semantics::SemanticAnalyzer::new(&text);
        analyzer.analyze(&ast)?;

        Ok(())
    }

    #[test]
    fn test_invalid_file_extension() {
        assert_eq!(
            analyze("file.txt\n└── when something").unwrap_err(),
            vec![
                semantics::Error {
                    kind: FileExtensionInvalid,
                    text: "file.txt\n└── when something".to_string(),
                    span: Span::new(Position::new(0, 1, 1), Position::new(7, 1, 8)),
                },
                semantics::Error {
                    kind: ConditionEmpty,
                    text: "file.txt\n└── when something".to_string(),
                    span: Span::new(Position::new(9, 2, 1), Position::new(32, 2, 18)),
                },
            ]
        );
    }

    #[test]
    fn test_invalid_extension_empty_tree() {
        assert_eq!(
            analyze("file.sol").unwrap_err(),
            vec![semantics::Error {
                kind: TreeEmpty,
                text: "file.sol".to_string(),
                span: Span::new(Position::new(7, 1, 8), Position::new(7, 1, 8)),
            }]
        );
        assert_eq!(
            analyze("file.txt").unwrap_err(),
            vec![
                semantics::Error {
                    kind: FileExtensionInvalid,
                    text: "file.txt".to_string(),
                    span: Span::new(Position::new(0, 1, 1), Position::new(7, 1, 8)),
                },
                semantics::Error {
                    kind: TreeEmpty,
                    text: "file.txt".to_string(),
                    span: Span::new(Position::new(7, 1, 8), Position::new(7, 1, 8)),
                }
            ]
        );
    }

    #[test]
    fn test_unexpected_node() {
        let ast = ast::Ast::Root(ast::Root {
            file_name: "file.sol".to_string(),
            asts: vec![ast::Ast::Root(ast::Root {
                file_name: "file.sol".to_string(),
                asts: vec![],
                span: Span::new(Position::new(0, 1, 1), Position::new(7, 1, 8)),
            })],
            span: Span::new(Position::new(0, 1, 1), Position::new(7, 1, 8)),
        });

        let mut analyzer = semantics::SemanticAnalyzer::new("file.sol");
        let result = analyzer.analyze(&ast);
        assert_eq!(
            result.unwrap_err(),
            vec![semantics::Error {
                kind: NodeUnexpected,
                text: "file.sol".to_string(),
                span: Span::new(Position::new(0, 1, 1), Position::new(7, 1, 8)),
            }]
        );
    }

    #[test]
    fn test_condition_empty() {
        assert_eq!(
            analyze("file.sol\n└── when something").unwrap_err(),
            vec![semantics::Error {
                kind: ConditionEmpty,
                text: "file.sol\n└── when something".to_string(),
                span: Span::new(Position::new(9, 2, 1), Position::new(32, 2, 18)),
            }]
        );
    }

    #[test]
    fn test_action_without_conditions() {
        assert_eq!(
            analyze("file.sol\n└── it a something").unwrap_err(),
            vec![semantics::Error {
                kind: ActionWithoutConditions,
                text: "file.sol\n└── it a something".to_string(),
                span: Span::new(Position::new(9, 2, 1), Position::new(32, 2, 18)),
            }]
        );
    }
}
