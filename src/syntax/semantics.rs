//! Implementation of the semantic analysis of a bulloak tree.

use std::collections::HashMap;
use std::{fmt, result};

use super::ast::{self, Ast};
use crate::span::Span;
use crate::syntax::visitor::Visitor;
use crate::utils::{lower_first_letter, to_pascal_case};

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
    #[must_use]
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// The original text string in which this error occurred.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the span at which this error occurred.
    #[must_use]
    pub fn span(&self) -> &Span {
        &self.span
    }

    #[cfg(test)]
    pub fn new(kind: ErrorKind, text: String, span: Span) -> Self {
        Error { kind, text, span }
    }
}

impl std::error::Error for Error {}

/// The type of an error that occurred while building an AST.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Found two conditions with the same title.
    ConditionDuplicated(Vec<Span>),
    /// Found a condition with no children.
    ConditionEmpty,
    /// Found an unexpected node. This is most probably a bug in the
    /// parser implementation.
    NodeUnexpected,
    /// Found no rules to emit.
    TreeEmpty,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::error::Formatter::from(self).fmt(f)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ErrorKind::{ConditionDuplicated, ConditionEmpty, NodeUnexpected, TreeEmpty};
        match *self {
            ConditionDuplicated(ref spans) => {
                let lines = spans
                    .iter()
                    .map(|s| s.start.line.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "found a condition more than once in lines: {}", lines)
            }
            ConditionEmpty => write!(f, "found a condition with no children"),
            NodeUnexpected => write!(f, "unexpected child node"),
            TreeEmpty => write!(f, "no rules where defined"),
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
    /// A map from modifier name to it's locations in the input.
    modifiers: HashMap<String, Vec<Span>>,
}

impl<'t> SemanticAnalyzer<'t> {
    /// Create a new semantic analyzer.
    #[must_use]
    pub fn new(text: &'t str) -> SemanticAnalyzer {
        SemanticAnalyzer {
            text,
            errors: Vec::new(),
            modifiers: HashMap::new(),
        }
    }

    /// Create a new error given an AST node and error type.
    fn error(&mut self, span: Span, kind: ErrorKind) {
        self.errors.push(Error {
            kind,
            text: self.text.to_owned(),
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
            Ast::ActionDescription(description) => self.visit_description(description),
        }
        // It is fine to unwrap here since analysis errors will
        // be stored in `self.errors`.
        .unwrap();

        // Check for duplicate conditions.
        for spans in self.modifiers.clone().into_values() {
            if spans.len() > 1 {
                self.error(
                    // FIXME: This is a patch until we start storing locations for
                    // parts of an AST node. In this case, we need the location of
                    // the condition's title.
                    spans[0].with_end(spans[0].start),
                    ErrorKind::ConditionDuplicated(spans),
                );
            }
        }

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
        if root.children.is_empty() {
            self.error(Span::splat(root.span.end), ErrorKind::TreeEmpty);
        }

        for ast in &root.children {
            match ast {
                Ast::Condition(condition) => {
                    self.visit_condition(condition)?;
                }
                Ast::Action(action) => {
                    self.visit_action(action)?;
                }
                node => {
                    self.error(*node.span(), ErrorKind::NodeUnexpected);
                }
            }
        }

        Ok(())
    }

    fn visit_condition(
        &mut self,
        condition: &ast::Condition,
    ) -> result::Result<Self::Output, Self::Error> {
        if condition.children.is_empty() {
            self.error(condition.span, ErrorKind::ConditionEmpty);
        }

        let modifier = lower_first_letter(&to_pascal_case(&condition.title));
        match self.modifiers.get_mut(&modifier) {
            Some(spans) => spans.push(condition.span),
            None => {
                self.modifiers.insert(modifier, vec![condition.span]);
            }
        }

        for ast in &condition.children {
            match ast {
                Ast::Condition(condition) => {
                    self.visit_condition(condition)?;
                }
                Ast::Action(action) => {
                    self.visit_action(action)?;
                }
                node => {
                    self.error(*node.span(), ErrorKind::NodeUnexpected);
                }
            }
        }

        Ok(())
    }

    fn visit_action(&mut self, _action: &ast::Action) -> result::Result<Self::Output, Self::Error> {
        // We don't implement any semantic rules here for now.
        Ok(())
    }

    fn visit_description(
        &mut self,
        _description: &ast::Description,
    ) -> result::Result<Self::Output, Self::Error> {
        // We don't implement any semantic rules here for now.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::span::{Position, Span};
    use crate::syntax::ast;
    use crate::syntax::parser::Parser;
    use crate::syntax::semantics::{self, ErrorKind::*};
    use crate::syntax::tokenizer::Tokenizer;

    fn analyze(text: &str) -> semantics::Result<()> {
        let tokens = Tokenizer::new().tokenize(text).unwrap();
        let asts = Parser::new().parse(text, &tokens).unwrap();
        let mut analyzer = semantics::SemanticAnalyzer::new(&text);
        asts.iter().try_for_each(|ast| analyzer.analyze(ast))?;

        Ok(())
    }

    #[test]
    fn unexpected_node() {
        let ast = ast::Ast::Root(ast::Root {
            contract_name: "Foo_Test".to_owned(),
            children: vec![ast::Ast::Root(ast::Root {
                contract_name: "Foo_Test".to_owned(),
                children: vec![],
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
                text: "Foo_Test".to_owned(),
                span: Span::new(Position::new(0, 1, 1), Position::new(7, 1, 8)),
            }]
        );
    }

    #[test]
    fn duplicated_condition() {
        assert_eq!(
            analyze(
            "Foo_Test
├── when dup
│   └── It 1
├── when dup
│   └── It 2
└── when dup
    └── It 3",
        )
        .unwrap_err(),
            vec![semantics::Error {
                kind: ConditionDuplicated(
                    vec![
                        Span::new(Position::new(9, 2, 1), Position::new(47, 3, 12)),
                        Span::new(Position::new(49, 4, 1), Position::new(87, 5, 12)),
                        Span::new(Position::new(89, 6, 1), Position::new(125, 7, 12)),
                    ],
                ),
                text: "Foo_Test\n├── when dup\n│   └── It 1\n├── when dup\n│   └── It 2\n└── when dup\n    └── It 3".to_owned(),
                span: Span::new(Position::new(9, 2, 1), Position::new(9, 2, 1)),
            }]
        );
    }

    #[test]
    fn condition_empty() {
        assert_eq!(
            analyze("Foo_Test\n└── when something").unwrap_err(),
            vec![semantics::Error {
                kind: ConditionEmpty,
                text: "Foo_Test\n└── when something".to_owned(),
                span: Span::new(Position::new(9, 2, 1), Position::new(32, 2, 18)),
            }]
        );
    }

    #[test]
    fn allow_action_without_conditions() {
        assert!(analyze("Foo_Test\n└── it a something").is_ok());
    }
}
