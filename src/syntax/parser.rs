//! A parser implementation for a stream of tokens representing a bulloak tree.

use std::fmt;
use std::{borrow::Borrow, cell::Cell, result};

use super::ast::{Action, Ast, Condition, Description, Root};
use super::tokenizer::{Token, TokenKind};
use crate::span::Span;
use crate::utils::{repeat_str, sanitize};

type Result<T> = result::Result<T, Error>;

/// An error that occurred while parsing a sequence of tokens into an abstract
/// syntax tree (AST).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    /// The kind of error.
    kind: ErrorKind,
    /// The original text that the parser generated the error from. Every
    /// span in an error is a valid range into this string.
    text: String,
    /// The span of this error.
    span: Span,
}

impl std::error::Error for Error {}

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
}

type Lexeme = String;

/// The type of an error that occurred while building an AST.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// This might happen because of an internal bug or the user
    /// might have passed an invalid .tree.
    /// An example of how this might be an internal bug is if the
    /// parser ends up in a state where the current grammar production
    /// being applied doesn't expect this token to occur.
    TokenUnexpected(Lexeme),
    /// Did not expect this token when parsing a description node.
    DescriptionTokenUnexpected(Lexeme),
    /// Did not expect this When keyword.
    WhenUnexpected,
    /// Did not expect this Given keyword.
    GivenUnexpected,
    /// Did not expect this It keyword.
    ItUnexpected,
    /// Did not expect a Word.
    WordUnexpected(Lexeme),
    /// Did not expect an end of file.
    EofUnexpected,
    /// The token stream was empty, so the tree is empty.
    TreeEmpty,
    /// A condition or action with no title was found.
    TitleMissing,
    /// A tree without a root was found.
    TreeRootless,
    /// A corner is not the last child.
    CornerNotLastChild,
    /// A tee is the last child.
    TeeLastChild,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::error::Formatter::from(self).fmt(f)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ErrorKind::{
            CornerNotLastChild, DescriptionTokenUnexpected, EofUnexpected, GivenUnexpected,
            ItUnexpected, TeeLastChild, TitleMissing, TokenUnexpected, TreeEmpty, TreeRootless,
            WhenUnexpected, WordUnexpected,
        };
        match self {
            TokenUnexpected(lexeme) => write!(f, "unexpected token '{lexeme}'"),
            DescriptionTokenUnexpected(lexeme) => {
                write!(f, "unexpected token in description '{lexeme}'")
            }
            WhenUnexpected => write!(f, "unexpected `when` keyword"),
            GivenUnexpected => write!(f, "unexpected `given` keyword"),
            ItUnexpected => write!(f, "unexpected `it` keyword"),
            WordUnexpected(lexeme) => write!(f, "unexpected `word` '{lexeme}'"),
            EofUnexpected => write!(f, "unexpected end of file"),
            TreeEmpty => write!(f, "found an empty tree"),
            TreeRootless => write!(f, "missing a root"),
            TitleMissing => write!(f, "found a condition/action without a title"),
            CornerNotLastChild => write!(f, "a `Corner` must be the last child"),
            TeeLastChild => write!(f, "a `Tee` must not be the last child"),
        }
    }
}

/// A parser for a sequence of .tree tokens into an abstract syntax tree (AST).
///
/// This struct represents the state of the parser. It is not
/// tied to any particular input, while `ParserI` is.
#[derive(Clone, Default)]
pub struct Parser {
    /// The index of the current token.
    current: Cell<usize>,
}

impl Parser {
    /// Create a new parser.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current: Cell::new(0),
        }
    }

    /// Parse the given tokens into an abstract syntax tree (AST).
    ///
    /// `parse` is the entry point for the parser. It takes a sequence of
    /// tokens and returns an AST.
    pub fn parse(&mut self, text: &str, tokens: &[Token]) -> Result<Ast> {
        ParserI::new(self, text, tokens).parse()
    }

    /// Reset the parser to its initial state.
    fn reset(&self) {
        self.current.set(0);
    }
}

/// The internal implementation of the parser.
struct ParserI<'t, P> {
    /// The input text.
    text: &'t str,
    /// The sequence of tokens to parse.
    tokens: &'t [Token],
    /// The parser state.
    parser: P,
}

impl<'t, P: Borrow<Parser>> ParserI<'t, P> {
    /// Create a new parser given the parser state, input text, and tokens.
    const fn new(parser: P, text: &'t str, tokens: &'t [Token]) -> Self {
        Self {
            text,
            tokens,
            parser,
        }
    }

    /// Return a reference to the state of the parser.
    fn parser(&self) -> &Parser {
        self.parser.borrow()
    }

    /// Create a new error with the given span and error type.
    fn error(&self, span: Span, kind: ErrorKind) -> Error {
        Error {
            kind,
            text: self.text.to_owned(),
            span,
        }
    }

    /// Returns true if the next call to `current` would
    /// return `None`.
    fn is_eof(&self) -> bool {
        self.parser().current.get() == self.tokens.len()
    }

    /// Return the current token.
    ///
    /// Returns `None` if the parser is past the end
    /// of the token stream.
    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.parser().current.get())
    }

    /// Return a reference to the next token.
    ///
    /// Returns `None` if the parser is currently at, or
    /// past the end of the token stream.
    fn peek(&self) -> Option<&Token> {
        let current_index = self.parser().current.get();
        self.tokens.get(current_index + 1)
    }

    /// Return the previous token.
    ///
    /// Returns `None` if the parser is currently at the start
    /// of the token stream.
    fn previous(&self) -> Option<&Token> {
        match self.parser().current.get() {
            0 => None,
            current => self.tokens.get(current - 1),
        }
    }

    /// Move to the next token, returning a reference to it.
    ///
    /// If there are no more tokens, return `None`.
    fn consume(&self) -> Option<&Token> {
        if self.is_eof() {
            return None;
        }
        self.parser().current.set(self.parser().current.get() + 1);
        self.tokens.get(self.parser().current.get())
    }

    /// Parse the given tokens into an abstract syntax tree.
    ///
    /// This is the entry point for the parser. Note that
    /// this method resets the parser state before parsing and
    /// that we defer the implementation of parsing to `_parse`.
    pub(crate) fn parse(&self) -> Result<Ast> {
        self.parser().reset();

        let root_token = self
            .current()
            .ok_or(self.error(Span::default(), ErrorKind::TreeEmpty))?;

        match root_token.kind {
            TokenKind::Word => self.parse_root(root_token),
            _ => Err(self.error(root_token.span, ErrorKind::TreeRootless)),
        }
    }

    /// Parse the root node of the AST.
    ///
    /// A root has the form:
    /// ```grammar
    /// CONTRACT_NAME
    /// (<TEE> [Condition | Action])*
    /// <CORNER> [Condition | Action]
    /// ```
    ///
    /// Panics if called when the parser is not at a `Word` token.
    fn parse_root(&self, token: &Token) -> Result<Ast> {
        assert!(matches!(token.kind, TokenKind::Word));
        self.consume();

        // The loop invariant is that `self.current` is a
        // `Tee` or the last `Corner`.
        let mut children = vec![];
        while let Some(current_token) = self.current() {
            let child = match current_token.kind {
                TokenKind::Corner | TokenKind::Tee => self.parse_branch(current_token)?,
                TokenKind::Word => Err(self.error(
                    current_token.span,
                    ErrorKind::WordUnexpected(current_token.lexeme.clone()),
                ))?,
                TokenKind::When => Err(self.error(current_token.span, ErrorKind::WhenUnexpected))?,
                TokenKind::Given => {
                    Err(self.error(current_token.span, ErrorKind::GivenUnexpected))?
                }
                TokenKind::It => Err(self.error(current_token.span, ErrorKind::ItUnexpected))?,
            };

            children.push(child);
        }

        let last_span = if children.is_empty() {
            &token.span
        } else {
            children.iter().last().unwrap().span()
        };

        Ok(Ast::Root(Root {
            span: Span::new(token.span.start, last_span.end),
            children,
            contract_name: token.lexeme.clone(),
        }))
    }

    /// Parse a branch.
    ///
    /// A branch is a production that starts with a `Tee` or a `Corner`
    /// token.
    ///
    /// Panics if called when the parser is not at a `Tee` or a `Corner`
    /// token.
    fn parse_branch(&self, token: &Token) -> Result<Ast> {
        assert!(matches!(token.kind, TokenKind::Tee | TokenKind::Corner));

        let first_token = self.peek().ok_or(self.error(
            token.span.with_start(token.span.end),
            ErrorKind::EofUnexpected,
        ))?;

        let ast = match first_token.kind {
            TokenKind::When | TokenKind::Given => self.parse_condition(token)?,
            TokenKind::It => self.parse_action(token)?,
            _ => Err(self.error(
                first_token.span,
                ErrorKind::TokenUnexpected(first_token.lexeme.clone()),
            ))?,
        };

        if matches!(token.kind, TokenKind::Tee) && self.is_eof() {
            return Err(self.error(
                token.span.with_start(token.span.end),
                ErrorKind::TeeLastChild,
            ));
        } else if matches!(token.kind, TokenKind::Corner) && !self.is_eof() {
            return Err(self.error(
                token.span.with_start(token.span.end),
                ErrorKind::CornerNotLastChild,
            ));
        };

        Ok(ast)
    }

    /// Parse a condition node.
    ///
    /// A condition has the form:
    /// ```grammar
    /// (<TEE> | <CORNER>) (<WHEN> | <GIVEN>) <WORD>*
    ///   (<TEE> [Condition | Action])*
    ///   <CORNER> [Condition | Action]
    /// ```
    ///
    /// Panics if called when the parser is not at a `Tee` or a `Corner`
    /// token.
    fn parse_condition(&self, token: &Token) -> Result<Ast> {
        assert!(matches!(token.kind, TokenKind::Tee | TokenKind::Corner));

        let start_token = self.peek().ok_or(self.error(
            token.span.with_start(token.span.end),
            ErrorKind::EofUnexpected,
        ))?;
        let title = self.parse_string(start_token);

        if title.len() == start_token.lexeme.len() {
            return Err(self.error(start_token.span, ErrorKind::TitleMissing));
        };

        let mut children = vec![];
        while self
            .current()
            // Only parse tokens that are indented more than the current token.
            // The column determines the tree level we are in.
            .is_some_and(|t| t.span.start.column > token.span.start.column)
        {
            let next_token = self.peek().ok_or(self.error(
                token.span.with_start(token.span.end),
                ErrorKind::EofUnexpected,
            ))?;

            let current_token = self.current().unwrap();
            let ast = match next_token.kind {
                TokenKind::When | TokenKind::Given => self.parse_condition(current_token)?,
                TokenKind::It => self.parse_action(current_token)?,
                _ => Err(self.error(
                    next_token.span,
                    ErrorKind::TokenUnexpected(next_token.lexeme.clone()),
                ))?,
            };

            children.push(ast);
        }

        let previous = self.previous().unwrap();
        Ok(Ast::Condition(Condition {
            title: sanitize(&title),
            children,
            span: Span::new(token.span.start, previous.span.end),
        }))
    }

    /// Parse an action node.
    ///
    /// An action has the form:
    /// ```grammar
    /// (<TEE> | <CORNER>) <IT> <WORD>*
    ///   (<TEE> ActionDescription)*
    ///   <CORNER> ActionDescription
    /// ```
    ///
    /// Panics if called when the parser is not at a `Tee` or a `Corner`
    /// token.
    fn parse_action(&self, token: &Token) -> Result<Ast> {
        assert!(matches!(token.kind, TokenKind::Tee | TokenKind::Corner));

        let start_token = self.peek().ok_or(self.error(
            token.span.with_start(token.span.end),
            ErrorKind::EofUnexpected,
        ))?;
        let title = self.parse_string(start_token);

        let mut children = vec![];
        while self
            .current()
            // Only parse tokens that are indented more than the current token.
            // The column determines the tree level we are in.
            .is_some_and(|t| t.span.start.column > token.span.start.column)
        {
            let next_token = self.peek().ok_or(self.error(
                token.span.with_start(token.span.end),
                ErrorKind::EofUnexpected,
            ))?;

            let current_token = self.current().unwrap();
            let ast = match next_token.kind {
                TokenKind::Word => self.parse_description(
                    current_token,
                    current_token.span.start.column - token.span.start.column,
                )?,
                _ => Err(self.error(
                    next_token.span,
                    ErrorKind::DescriptionTokenUnexpected(next_token.lexeme.clone()),
                ))?,
            };

            children.push(ast);
        }

        let previous = self.previous().unwrap();
        Ok(Ast::Action(Action {
            title,
            children,
            span: Span::new(token.span.start, previous.span.end),
        }))
    }

    /// Parse an action description node.
    ///
    /// An action description has the form:
    /// ```grammar
    /// [<TEE> <WORD>* | <CORNER> <WORD>]
    ///   (<TEE> ActionDescription)*
    ///   <CORNER> ActionDescription
    /// ```
    ///
    /// This function receives a `column_delta` used to know
    /// the number of spaces to prepend the lexeme with. E.g.
    /// For the following action:
    ///
    /// ```tree
    /// It should do something.
    ///     <CORNER> I describe the above action.
    /// ^^^^
    /// ```
    ///
    /// Then, `column_delta = 4` and the emitted description should
    /// respect this.
    ///
    /// Panics if called when the parser is not at a `Tee` or a `Corner`
    /// token.
    fn parse_description(&self, token: &Token, column_delta: usize) -> Result<Ast> {
        assert!(matches!(token.kind, TokenKind::Tee | TokenKind::Corner));

        let start_token = self.peek().ok_or(self.error(
            token.span.with_start(token.span.end),
            ErrorKind::EofUnexpected,
        ))?;
        let text = self.parse_string(start_token);

        let previous = self.previous().unwrap();
        Ok(Ast::ActionDescription(Description {
            text: format!("{}{}", repeat_str(" ", column_delta), text),
            span: Span::new(token.span.start, previous.span.end),
        }))
    }

    /// Parse a string.
    ///
    /// A string is a sequence of words separated by spaces.
    ///
    /// Consumes all the tokens including the given token until no more words
    /// are found.
    fn parse_string(&self, start_token: &Token) -> String {
        self.consume();
        let mut string = String::from(&start_token.lexeme);

        // Consume all words.
        while let Some(token) = self.consume() {
            match token.kind {
                TokenKind::Word | TokenKind::It | TokenKind::When | TokenKind::Given => {
                    string = string + " " + &token.lexeme;
                }
                _ => break,
            }
        }

        string
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::span::Span;
    use crate::syntax::ast::{Action, Ast, Condition, Description, Root};
    use crate::syntax::parser::{self, ErrorKind, Parser};
    use crate::syntax::test_utils::{p, s, TestError};
    use crate::syntax::tokenizer::Tokenizer;

    impl PartialEq<parser::Error> for TestError<parser::ErrorKind> {
        fn eq(&self, other: &parser::Error) -> bool {
            self.span == other.span && self.kind == other.kind
        }
    }

    impl PartialEq<TestError<parser::ErrorKind>> for parser::Error {
        fn eq(&self, other: &TestError<parser::ErrorKind>) -> bool {
            self.span == other.span && self.kind == other.kind
        }
    }

    fn e<K>(kind: K, span: Span) -> TestError<K> {
        TestError { kind, span }
    }

    fn parse(file_contents: &str) -> parser::Result<Ast> {
        let tokens = Tokenizer::new().tokenize(file_contents).unwrap();
        Parser::new().parse(file_contents, &tokens)
    }

    #[test]
    fn empty_tree() {
        assert_eq!(
            parse("").unwrap_err(),
            e(ErrorKind::TreeEmpty, Span::default())
        );
    }

    #[test]
    fn rootless_tree() {
        assert_eq!(
            parse("└── It should never revert.").unwrap_err(),
            e(ErrorKind::TreeRootless, Span::default())
        );
        assert_eq!(
            parse("├── It should revert.").unwrap_err(),
            e(ErrorKind::TreeRootless, Span::default())
        );
        assert_eq!(
            parse("└── When stuff happens").unwrap_err(),
            e(ErrorKind::TreeRootless, Span::default())
        );
        assert_eq!(
            parse("├── When stuff happens").unwrap_err(),
            e(ErrorKind::TreeRootless, Span::default())
        );
        assert_eq!(
            parse("└── this is a description").unwrap_err(),
            e(ErrorKind::TreeRootless, Span::default())
        );
    }

    #[test]
    fn tee_last_child_errors() {
        assert_eq!(
            parse("Foo_Test\n├── when something bad happens\n   └── it should revert").unwrap_err(),
            e(ErrorKind::TeeLastChild, Span::splat(p(9, 2, 1)))
        );
    }

    #[test]
    fn corner_not_last_child_errors() {
        assert_eq!(
            parse(
                r"Foo_Test
└── when something bad happens
   └── it should revert
└── when something happens
   └── it should not revert"
            )
            .unwrap_err(),
            e(ErrorKind::CornerNotLastChild, Span::splat(p(9, 2, 1)))
        );
    }

    #[test]
    fn only_contract_name() {
        assert_eq!(
            parse("FooTest").unwrap(),
            Ast::Root(Root {
                span: s(p(0, 1, 1), p(6, 1, 7)),
                children: vec![],
                contract_name: String::from("FooTest"),
            })
        );
    }

    #[test]
    fn one_child() {
        assert_eq!(
            parse("Foo_Test\n└── when something bad happens\n   └── it should revert").unwrap(),
            Ast::Root(Root {
                contract_name: String::from("Foo_Test"),
                span: s(p(0, 1, 1), p(74, 3, 23)),
                children: vec![Ast::Condition(Condition {
                    span: s(p(9, 2, 1), p(74, 3, 23)),
                    title: String::from("when something bad happens"),
                    children: vec![Ast::Action(Action {
                        span: s(p(49, 3, 4), p(74, 3, 23)),
                        title: String::from("it should revert"),
                        children: vec![]
                    })],
                })],
            })
        );
    }

    #[test]
    fn one_action_description() {
        assert_eq!(
            parse(
                r"Foo_Test
└── when something bad happens
   └── it should revert
      └── because _bad_"
            )
            .unwrap(),
            Ast::Root(Root {
                span: s(p(0, 1, 1), p(104, 4, 23)),
                children: vec![Ast::Condition(Condition {
                    span: s(p(9, 2, 1), p(104, 4, 23)),
                    title: String::from("when something bad happens"),
                    children: vec![Ast::Action(Action {
                        span: s(p(49, 3, 4), p(104, 4, 23)),
                        title: String::from("it should revert"),
                        children: vec![Ast::ActionDescription(Description {
                            span: s(p(82, 4, 7), p(104, 4, 23)),
                            text: String::from("   because _bad_"),
                        })]
                    })],
                })],
                contract_name: String::from("Foo_Test"),
            })
        );
    }

    #[test]
    fn nested_action_descriptions() {
        assert_eq!(
            parse(
                r"Foo_Test
└── when something bad happens
   └── it should revert
      ├── some stuff happened
      │  └── and that stuff
      └── was very _bad_"
            )
            .unwrap(),
            Ast::Root(Root {
                span: s(p(0, 1, 1), p(177, 6, 24)),
                children: vec![Ast::Condition(Condition {
                    span: s(p(9, 2, 1), p(177, 6, 24)),
                    title: String::from("when something bad happens"),
                    children: vec![Ast::Action(Action {
                        span: s(p(49, 3, 4), p(177, 6, 24)),
                        title: String::from("it should revert"),
                        children: vec![
                            Ast::ActionDescription(Description {
                                span: s(p(82, 4, 7), p(110, 4, 29)),
                                text: String::from("   some stuff happened"),
                            }),
                            Ast::ActionDescription(Description {
                                span: s(p(123, 5, 10), p(146, 5, 27)),
                                text: String::from("      and that stuff"),
                            }),
                            Ast::ActionDescription(Description {
                                span: s(p(154, 6, 7), p(177, 6, 24)),
                                text: String::from("   was very _bad_"),
                            }),
                        ]
                    })],
                })],
                contract_name: String::from("Foo_Test"),
            })
        );
    }

    #[test]
    fn unexpected_tokens() {
        use ErrorKind::*;
        assert_eq!(
            parse(r"a └ └").unwrap_err(),
            e(TokenUnexpected("└".to_owned()), Span::splat(p(6, 1, 5)))
        );
        assert_eq!(
            parse(r"a ├ ├").unwrap_err(),
            e(TokenUnexpected("├".to_owned()), Span::splat(p(6, 1, 5)))
        );
        assert_eq!(
            parse(r"a └").unwrap_err(),
            e(EofUnexpected, Span::splat(p(2, 1, 3)))
        );
        assert_eq!(
            parse(r"a └ when").unwrap_err(),
            e(TitleMissing, s(p(6, 1, 5), p(9, 1, 8)))
        );
        assert_eq!(
            parse(r"a ├").unwrap_err(),
            e(EofUnexpected, Span::splat(p(2, 1, 3)))
        );
        assert_eq!(
            parse(r"a when").unwrap_err(),
            e(WhenUnexpected, s(p(2, 1, 3), p(5, 1, 6)))
        );
        assert_eq!(
            parse(r"a given").unwrap_err(),
            e(GivenUnexpected, s(p(2, 1, 3), p(6, 1, 7)))
        );
        assert_eq!(
            parse(r"a it").unwrap_err(),
            e(ItUnexpected, s(p(2, 1, 3), p(3, 1, 4)))
        );
        assert_eq!(
            parse(r"a b").unwrap_err(),
            e(WordUnexpected("b".to_owned()), Span::splat(p(2, 1, 3)))
        );
    }

    #[test]
    fn descriptions_are_the_only_action_children() {
        assert_eq!(
            parse(
                r"Foo_Test
└── when something bad happens
   └── it should revert
      └── it because _bad_"
            )
            .unwrap_err(),
            e(
                ErrorKind::DescriptionTokenUnexpected("it".to_owned()),
                s(p(92, 4, 11), p(93, 4, 12))
            )
        );
    }

    #[test]
    fn two_children() {
        assert_eq!(
            parse(
                r"FooBarTheBest_Test
├── when stuff called
│  └── it should revert
└── given not stuff called
   └── it should revert"
            )
            .unwrap(),
            Ast::Root(Root {
                contract_name: String::from("FooBarTheBest_Test"),
                span: s(p(0, 1, 1), p(140, 5, 23)),
                children: vec![
                    Ast::Condition(Condition {
                        title: String::from("when stuff called"),
                        span: s(p(19, 2, 1), p(77, 3, 23)),
                        children: vec![Ast::Action(Action {
                            title: String::from("it should revert"),
                            span: s(p(52, 3, 4), p(77, 3, 23)),
                            children: vec![]
                        })],
                    }),
                    Ast::Condition(Condition {
                        title: String::from("given not stuff called"),
                        span: s(p(79, 4, 1), p(140, 5, 23)),
                        children: vec![Ast::Action(Action {
                            title: String::from("it should revert"),
                            span: s(p(115, 5, 4), p(140, 5, 23)),
                            children: vec![]
                        })],
                    }),
                ],
            })
        );
    }

    // https://github.com/alexfertel/bulloak/issues/54
    #[test]
    fn parses_top_level_actions() {
        assert_eq!(
            parse(
                r#"Foo
└── It reverts when X."#
            )
            .unwrap(),
            Ast::Root(Root {
                contract_name: String::from("Foo"),
                span: s(p(0, 1, 1), p(31, 2, 22)),
                children: vec![Ast::Action(Action {
                    title: String::from("It reverts when X."),
                    span: s(p(4, 2, 1), p(31, 2, 22)),
                    children: vec![]
                })],
            })
        );
    }

    #[test]
    fn unsanitized_input() {
        assert_eq!(
            parse(
                r#"FooB-rTheBestOf_Test
└── when st-ff "all'd
   └── it should revert"#
            )
            .unwrap(),
            Ast::Root(Root {
                contract_name: String::from("FooB-rTheBestOf_Test"),
                span: s(p(0, 1, 1), p(77, 3, 23)),
                children: vec![Ast::Condition(Condition {
                    title: String::from("when st_ff alld"),
                    span: s(p(21, 2, 1), p(77, 3, 23)),
                    children: vec![Ast::Action(Action {
                        title: String::from("it should revert"),
                        span: s(p(52, 3, 4), p(77, 3, 23)),
                        children: vec![]
                    })],
                })],
            })
        );
    }
}
