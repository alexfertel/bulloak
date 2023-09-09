use std::fmt;
use std::{borrow::Borrow, cell::Cell, result};

use super::ast::{Action, Ast, Condition, Root};
use super::span::Span;
use super::tokenizer::{Token, TokenKind};
use crate::utils::sanitize;

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

type Lexeme = String;

/// The type of an error that occurred while building an AST.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// This might happen because of an internal bug or the user
    /// might have passed an invalid .tree.
    /// An example of how this might be an internal bug is if the
    /// parser ends up in a state where the current grammar production
    /// being applied doesn't expect this token to occur.
    TokenUnexpected(Lexeme),
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
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        super::error::Formatter::from(self).fmt(f)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ErrorKind::*;
        match *self {
            TokenUnexpected(ref lexeme) => write!(f, "unexpected token: {}", lexeme),
            WhenUnexpected => write!(f, "unexpected `when` keyword"),
            GivenUnexpected => write!(f, "unexpected `given` keyword"),
            ItUnexpected => write!(f, "unexpected `it` keyword"),
            WordUnexpected(ref lexeme) => write!(f, "unexpected `word`: {}", lexeme),
            EofUnexpected => write!(f, "unexpected end of file"),
            _ => unreachable!(),
        }
    }
}

/// A parser for a sequence of .tree tokens into an abstract syntax tree (AST).
///
/// This struct represents the state of the parser. It is not
/// tied to any particular input, while `ParserI` is.
pub struct Parser {
    /// The index of the current token.
    current: Cell<usize>,
}

impl Parser {
    /// Create a new parser.
    pub fn new() -> Self {
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
    fn new(parser: P, text: &'t str, tokens: &'t [Token]) -> Self {
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
            text: self.text.to_string(),
            span,
        }
    }

    /// Return the current token.
    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.parser().current.get())
    }

    /// Return the previous token.
    ///
    /// Returns `None` if the parser is currently at the start
    /// of the token stream.
    fn previous(&self) -> Option<&Token> {
        if self.parser().current.get() == 0 {
            return None;
        }
        self.tokens.get(self.parser().current.get() - 1)
    }

    /// Move to the next token, returning a reference to it.
    ///
    /// If there are no more tokens, return `None`.
    fn consume(&self) -> Option<&Token> {
        if self.parser().current.get() == self.tokens.len() {
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
    pub fn parse(&self) -> Result<Ast> {
        self.parser().reset();
        self._parse()
    }

    /// Internal recursive implementation of parsing.
    ///
    /// The invariants of this method are:
    /// - The first call to this function parses the root node.
    /// - The parser is always at the start of a production when entering
    /// this function.
    fn _parse(&self) -> Result<Ast> {
        let current_token = match self.current() {
            Some(current) => current,
            None => {
                return Err(self.error(self.tokens.last().unwrap().span, ErrorKind::EofUnexpected))
            }
        };

        match current_token.kind {
            TokenKind::Word if self.parser().current.get() == 0 => self.parse_root(current_token),
            TokenKind::Tee | TokenKind::Corner => {
                let next_token = match self.consume() {
                    Some(next) => next,
                    None => {
                        return Err(
                            self.error(self.tokens.last().unwrap().span, ErrorKind::EofUnexpected)
                        )
                    }
                };

                match next_token.kind {
                    TokenKind::It => {
                        let title = self.parse_string(next_token);
                        let previous = self.previous().unwrap();
                        Ok(Ast::Action(Action {
                            title,
                            span: Span::new(current_token.span.start, previous.span.end),
                        }))
                    }
                    TokenKind::When | TokenKind::Given => {
                        let title = self.parse_string(next_token);

                        let mut asts = vec![];
                        while self
                            .current()
                            // Only parse tokens that are indented more than the current token.
                            // The column is our way to determine the tree level we are in.
                            .is_some_and(|t| t.span.start.column > current_token.span.start.column)
                        {
                            let ast = self._parse()?;
                            asts.push(ast);
                        }

                        let previous = self.previous().unwrap();
                        Ok(Ast::Condition(Condition {
                            title: sanitize(&title),
                            asts,
                            span: Span::new(current_token.span.start, previous.span.end),
                        }))
                    }
                    _ => Err(self.error(
                        current_token.span,
                        ErrorKind::TokenUnexpected(next_token.lexeme.clone()),
                    )),
                }
            }
            TokenKind::Word => Err(self.error(
                current_token.span,
                ErrorKind::WordUnexpected(current_token.lexeme.clone()),
            )),
            TokenKind::When => Err(self.error(current_token.span, ErrorKind::WhenUnexpected)),
            TokenKind::Given => Err(self.error(current_token.span, ErrorKind::GivenUnexpected)),
            TokenKind::It => Err(self.error(current_token.span, ErrorKind::ItUnexpected)),
        }
    }

    /// Parse the root node of the AST.
    fn parse_root(&self, current_token: &Token) -> Result<Ast> {
        self.consume();
        // A string at the start of the file is the root ast node.
        let mut asts = vec![];
        while self.current().is_some() {
            let ast = self._parse()?;
            asts.push(ast);
        }

        let last_span = if !asts.is_empty() {
            asts[asts.len() - 1].span()
        } else {
            &current_token.span
        };

        Ok(Ast::Root(Root {
            span: Span::new(current_token.span.start, last_span.end),
            asts,
            contract_name: current_token.lexeme.clone(),
        }))
    }

    /// Parse a string.
    ///
    /// A string is a sequence of words separated by spaces.
    fn parse_string(&self, start_token: &Token) -> String {
        // Strings always start with one of [It, When, Given].
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

    use crate::parse::ast::{Action, Ast, Condition, Root};
    use crate::parse::parser::{self, Parser};
    use crate::parse::span::{Position, Span};
    use crate::parse::tokenizer::Tokenizer;

    #[derive(Clone, Debug)]
    struct TestError {
        span: Span,
        kind: parser::ErrorKind,
    }

    impl PartialEq<parser::Error> for TestError {
        fn eq(&self, other: &parser::Error) -> bool {
            self.span == other.span && self.kind == other.kind
        }
    }

    impl PartialEq<TestError> for parser::Error {
        fn eq(&self, other: &TestError) -> bool {
            self.span == other.span && self.kind == other.kind
        }
    }

    fn parse(file_contents: &str) -> parser::Result<Ast> {
        let tokens = Tokenizer::new().tokenize(file_contents).unwrap();
        Parser::new().parse(file_contents, &tokens)
    }

    #[test]
    fn test_only_contract_name() {
        assert_eq!(
            parse("FooTest").unwrap(),
            Ast::Root(Root {
                span: Span::new(Position::new(0, 1, 1), Position::new(6, 1, 7)),
                asts: vec![],
                contract_name: String::from("FooTest"),
            })
        );
    }

    #[test]
    fn test_one_child() {
        assert_eq!(
            parse("Foo_Test\n└── when something bad happens\n   └── it should revert").unwrap(),
            Ast::Root(Root {
                span: Span::new(Position::new(0, 1, 1), Position::new(74, 3, 23)),
                asts: vec![Ast::Condition(Condition {
                    span: Span::new(Position::new(9, 2, 1), Position::new(74, 3, 23)),
                    title: String::from("when something bad happens"),
                    asts: vec![Ast::Action(Action {
                        span: Span::new(Position::new(49, 3, 4), Position::new(74, 3, 23)),
                        title: String::from("it should revert"),
                    })],
                })],
                contract_name: String::from("Foo_Test"),
            })
        );
    }

    #[test]
    fn test_two_children() {
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
                span: Span::new(Position::new(0, 1, 1), Position::new(140, 5, 23)),
                asts: vec![
                    Ast::Condition(Condition {
                        title: String::from("when stuff called"),
                        span: Span::new(Position::new(19, 2, 1), Position::new(77, 3, 23)),
                        asts: vec![Ast::Action(Action {
                            title: String::from("it should revert"),
                            span: Span::new(Position::new(52, 3, 4), Position::new(77, 3, 23)),
                        })],
                    }),
                    Ast::Condition(Condition {
                        title: String::from("given not stuff called"),
                        span: Span::new(Position::new(79, 4, 1), Position::new(140, 5, 23)),
                        asts: vec![Ast::Action(Action {
                            title: String::from("it should revert"),
                            span: Span::new(Position::new(115, 5, 4), Position::new(140, 5, 23)),
                        })],
                    }),
                ],
            })
        );
    }

    #[test]
    fn test_unsanitized_input() {
        assert_eq!(
            parse(
                r#"FooB-rTheBestOf_Test
└── when st-ff "all'd
   └── it should revert"#
            )
            .unwrap(),
            Ast::Root(Root {
                contract_name: String::from("FooB-rTheBestOf_Test"),
                span: Span::new(Position::new(0, 1, 1), Position::new(77, 3, 23)),
                asts: vec![Ast::Condition(Condition {
                    title: String::from("when st_ff alld"),
                    span: Span::new(Position::new(21, 2, 1), Position::new(77, 3, 23)),
                    asts: vec![Ast::Action(Action {
                        title: String::from("it should revert"),
                        span: Span::new(Position::new(52, 3, 4), Position::new(77, 3, 23)),
                    })],
                }),],
            })
        );
    }
}
