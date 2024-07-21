//! Implementation of the semantic analysis of a bulloak tree.
use std::{collections::HashMap, fmt, result};

use thiserror::Error;

use super::ast::{self, Ast};
use crate::{
    error::FrontendError,
    span::Span,
    utils::{lower_first_letter, sanitize, to_pascal_case},
    visitor::Visitor,
};

type Result<T> = result::Result<T, Errors>;

/// A collection of errors that occurred during semantic analysis.
#[derive(Error, Clone, Debug, PartialEq, Eq)]
#[error("{}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(""))]
pub struct Errors(pub Vec<Error>);

/// An error that occurred while doing semantic analysis on the abstract
/// syntax tree.
#[derive(Error, Clone, Debug, Eq, PartialEq)]
pub struct Error {
    /// The kind of error.
    #[source]
    kind: ErrorKind,
    /// The original text that the visitor generated the error from. Every
    /// span in an error is a valid range into this string.
    text: String,
    /// The span of this error.
    span: Span,
}

impl Error {
    #[cfg(test)]
    pub fn new(kind: ErrorKind, text: String, span: Span) -> Self {
        Error { kind, text, span }
    }
}

impl FrontendError<ErrorKind> for Error {
    /// Return the type of this error.
    fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// The original text string in which this error occurred.
    fn text(&self) -> &str {
        &self.text
    }

    /// Return the span at which this error occurred.
    fn span(&self) -> &Span {
        &self.span
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_error(f)
    }
}

fn format_spans(spans: &[Span]) -> String {
    spans
        .iter()
        .map(|s| s.start.line.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// The type of an error that occurred while building an AST.
#[derive(Error, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Found two conditions or top-level actions with the same title.
    #[error("found an identifier more than once in lines: {}", format_spans(.0))]
    IdentifierDuplicated(Vec<Span>),
    /// Found a condition with no children.
    #[error("found a condition with no children")]
    ConditionEmpty,
    /// Found an unexpected node. This is most probably a bug in the
    /// parser implementation.
    #[error("unexpected child node")]
    NodeUnexpected,
    /// Found no rules to emit.
    #[error("no rules where defined")]
    TreeEmpty,
}

/// A visitor that performs semantic analysis on an AST.
pub(crate) struct SemanticAnalyzer<'t> {
    /// A list of errors that occurred while analyzing the AST.
    errors: Vec<Error>,
    /// The original text that the visitor generated the errors from. Every
    /// span in an error is a valid range into this string.
    text: &'t str,
    /// A map from modifier name to it's locations in the input.
    identifiers: HashMap<String, Vec<Span>>,
}

impl<'t> SemanticAnalyzer<'t> {
    /// Create a new semantic analyzer.
    #[must_use]
    pub(crate) fn new(text: &'t str) -> SemanticAnalyzer {
        SemanticAnalyzer {
            text,
            errors: Vec::new(),
            identifiers: HashMap::new(),
        }
    }

    /// Create a new error given an AST node and error type.
    fn error(&mut self, span: Span, kind: ErrorKind) {
        self.errors.push(Error { kind, text: self.text.to_owned(), span });
    }

    /// Traverse the given AST and store any errors that occur.
    ///
    /// Note that this implementation is a bit weird in that we
    /// create the `Err` variant of the result by hand.
    pub(crate) fn analyze(&mut self, ast: &ast::Ast) -> Result<()> {
        match ast {
            Ast::Root(root) => self.visit_root(root),
            Ast::Condition(condition) => self.visit_condition(condition),
            Ast::Action(action) => self.visit_action(action),
            Ast::ActionDescription(description) => {
                self.visit_description(description)
            }
        }
        // It is fine to unwrap here since analysis errors will
        // be stored in `self.errors`.
        .unwrap();

        // Check for duplicate conditions.
        for spans in self.identifiers.clone().into_values() {
            if spans.len() > 1 {
                self.error(
                    // FIXME: This is a patch until we start storing locations
                    // for parts of an AST node. In this case, we need the
                    // location of the condition's title.
                    spans[0].with_end(spans[0].start),
                    ErrorKind::IdentifierDuplicated(spans),
                );
            }
        }

        if !self.errors.is_empty() {
            return Err(Errors(self.errors.clone()));
        }

        Ok(())
    }
}

/// A visitor that performs semantic analysis on an AST.
impl Visitor for SemanticAnalyzer<'_> {
    type Error = ();
    type Output = ();

    fn visit_root(
        &mut self,
        root: &ast::Root,
    ) -> result::Result<Self::Output, Self::Error> {
        if root.children.is_empty() {
            self.error(Span::splat(root.span.end), ErrorKind::TreeEmpty);
        }

        for ast in &root.children {
            match ast {
                Ast::Condition(condition) => {
                    self.visit_condition(condition)?;
                }
                Ast::Action(action) => {
                    // Top-level actions must be checked for duplicates since
                    // they will become Solidity functions.
                    let identifier = lower_first_letter(&to_pascal_case(
                        &sanitize(&action.title),
                    ));
                    match self.identifiers.get_mut(&identifier) {
                        Some(spans) => spans.push(action.span),
                        None => {
                            self.identifiers
                                .insert(identifier, vec![action.span]);
                        }
                    }
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

        let modifier =
            lower_first_letter(&to_pascal_case(&sanitize(&condition.title)));
        match self.identifiers.get_mut(&modifier) {
            Some(spans) => spans.push(condition.span),
            None => {
                self.identifiers.insert(modifier, vec![condition.span]);
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

    fn visit_action(
        &mut self,
        _action: &ast::Action,
    ) -> result::Result<Self::Output, Self::Error> {
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

    use crate::{
        ast,
        parser::Parser,
        semantics::{self, ErrorKind::*},
        span::{Position, Span},
        tokenizer::Tokenizer,
    };

    fn analyze(text: &str) -> semantics::Result<()> {
        let tokens = Tokenizer::new().tokenize(text).unwrap();
        let ast = Parser::new().parse(text, &tokens).unwrap();
        let mut analyzer = semantics::SemanticAnalyzer::new(&text);
        analyzer.analyze(&ast)?;

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
            result.unwrap_err().0,
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
        .unwrap_err().0,
            vec![semantics::Error {
                kind: IdentifierDuplicated(
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
    fn duplicated_top_level_action() {
        assert_eq!(
            analyze(
                "Foo_Test
├── It should, match the result.
└── It should' match the result.",
            )
            .unwrap_err()
            .0,
            vec![semantics::Error {
                kind: IdentifierDuplicated(vec![
                    Span::new(Position::new(9, 2, 1), Position::new(46, 2, 32)),
                    Span::new(Position::new(48, 3, 1), Position::new(85, 3, 32))
                ]),
                text:
                    "Foo_Test\n├── It should, match the result.\n└── It should' match the result."
                        .to_owned(),
                span: Span::new(Position::new(9, 2, 1), Position::new(9, 2, 1))
            }]
        );
    }

    #[test]
    fn condition_empty() {
        assert_eq!(
            analyze("Foo_Test\n└── when something").unwrap_err().0,
            vec![semantics::Error {
                kind: ConditionEmpty,
                text: "Foo_Test\n└── when something".to_owned(),
                span: Span::new(
                    Position::new(9, 2, 1),
                    Position::new(32, 2, 18)
                ),
            }]
        );
    }

    #[test]
    fn allow_action_without_conditions() {
        assert!(analyze("Foo_Test\n└── it a something").is_ok());
    }

    #[test]
    fn test_multiple_errors() {
        let text = r"test.sol
├── when 1
└── when 2"
            .to_owned();

        let errors = semantics::Errors(vec![
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
