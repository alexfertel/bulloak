//! Defines a scanner for bulloak trees that produces a token stream.

use std::{borrow::Borrow, cell::Cell, fmt, result};

use crate::span::{Position, Span};

type Result<T> = result::Result<T, Error>;

/// An error that occurred while tokenizing a .tree string into a sequence of
/// tokens.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    /// The kind of error.
    kind: ErrorKind,
    /// The original text that the tokenizer generated the error from. Every
    /// span in an error is a valid range into this string.
    text: String,
    /// The span of this error.
    span: Span,
}

impl std::error::Error for Error {}

impl Error {
    /// Return the type of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// The original text string in which this error occurred.
    ///
    /// Every span reported by this error is reported in terms of this string.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the span at which this error occurred.
    pub fn span(&self) -> &Span {
        &self.span
    }
}

/// The type of an error that occurred while tokenizing a tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// Found an invalid identifier character.
    IdentifierCharInvalid(char),
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
        match self {
            IdentifierCharInvalid(c) => write!(f, "invalid identifier: {:?}", c),
            _ => unreachable!(),
        }
    }
}

/// `Token` represents a single unit of meaning in a .tree.
///
/// A token has a kind, a span, and a lexeme. The kind is
/// the type of the token, the span is the range in which a
/// token appears in the original text, and the lexeme is the
/// text that the token represents.
#[derive(PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub lexeme: String,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Token({:?}, {:?}, {:?})",
            self.kind, self.lexeme, self.span
        )
    }
}

/// The type of a token.
#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// A token representing the `├` character.
    Tee,
    /// A token representing the `└` character.
    Corner,
    /// A token representing a string.
    ///
    /// For example, in the text `foo bar`, both `foo` and `bar` are
    /// `Word` tokens.
    Word,
    /// A token representing a `when` keyword.
    When,
    /// A token representing a `given` keyword.
    Given,
    /// A token representing an `it` keyword.
    It,
}

/// A tokenizer for .tree files.
///
/// This struct represents the state of the tokenizer. It is not
/// tied to any particular input, while `TokenizerI` is.
pub struct Tokenizer {
    /// The current position of the tokenizer in the input.
    ///
    /// By default this is set to the start of the input.
    pos: Cell<Position>,
    /// When true, the tokenizer is in `identifier` mode.
    ///
    /// In `identifier` mode, the tokenizer will error if it encounters a
    /// a character that is not a valid identifier character.
    /// This is to prevent malformed names when emitting identifiers.
    ///
    /// This is `true` by default because the first token must be
    /// a contract name, which has to be a valid solidity identifier.
    identifier_mode: Cell<bool>,
}

impl Tokenizer {
    /// Create a new tokenizer.
    pub fn new() -> Self {
        Self {
            pos: Cell::new(Position::new(0, 1, 1)),
            // Starts as `true` because the first token must always be a contract name.
            identifier_mode: Cell::new(true),
        }
    }

    /// Tokenize the input .tree text.
    ///
    /// `tokenize` is the entry point of the Tokenizer.
    /// It takes a string of .tree text and returns a vector of tokens.
    pub fn tokenize(&mut self, text: &str) -> Result<Vec<Token>> {
        TokenizerI::new(self, text).tokenize()
    }

    /// Reset the tokenizer's state.
    fn reset(&self) {
        self.pos.set(Position::new(0, 1, 1));
        self.identifier_mode.set(false);
    }
}

/// TokenizerI is the internal tokenizer implementation.
struct TokenizerI<'s, T> {
    /// The text being tokenized.
    text: &'s str,
    /// The tokenizer state.
    tokenizer: T,
}

impl<'s, T: Borrow<Tokenizer>> TokenizerI<'s, T> {
    /// Create an internal tokenizer from a tokenizer state holder
    /// and the input text.
    fn new(tokenizer: T, text: &'s str) -> Self {
        Self { text, tokenizer }
    }

    /// Return a reference to the tokenizer state.
    fn tokenizer(&self) -> &Tokenizer {
        self.tokenizer.borrow()
    }

    /// Create a new error with the given span and error type.
    fn error(&self, span: Span, kind: ErrorKind) -> Error {
        Error {
            kind,
            text: self.text.to_string(),
            span,
        }
    }

    /// Return a reference to the text being parsed.
    fn text(&self) -> &str {
        self.text.borrow()
    }

    /// Return the character at the current position of the tokenizer.
    ///
    /// This panics if the current position does not point to a valid char.
    fn char(&self) -> char {
        self.char_at(self.offset())
    }

    /// Return the character at the given position.
    ///
    /// This panics if the given position does not point to a valid char.
    fn char_at(&self, i: usize) -> char {
        self.text()[i..]
            .chars()
            .next()
            .unwrap_or_else(|| panic!("expected char at offset {}", i))
    }

    /// Return the current offset of the tokenizer.
    ///
    /// The offset starts at `0` from the beginning of the tree.
    fn offset(&self) -> usize {
        self.tokenizer().pos.get().offset
    }

    /// Returns true if the next call to `next` would return false.
    fn is_eof(&self) -> bool {
        self.offset() == self.text().len()
    }

    /// Return the current position of the tokenizer, which includes the offset,
    /// line and column.
    fn pos(&self) -> Position {
        self.tokenizer().pos.get()
    }

    /// Create a span at the current position of the tokenizer. Both the start
    /// and end of the span are set.
    fn span(&self) -> Span {
        Span::splat(self.pos())
    }

    /// Peek at the next character in the input without advancing the tokenizer.
    ///
    /// If the input has been exhausted, then this returns `None`.
    fn peek(&self) -> Option<char> {
        if self.is_eof() {
            return None;
        }
        self.text()[self.offset() + self.char().len_utf8()..]
            .chars()
            .next()
    }

    /// Enters identifier mode.
    fn enter_identifier_mode(&self) {
        self.tokenizer().identifier_mode.set(true);
    }

    /// Exits identifier mode.
    fn exit_identifier_mode(&self) {
        self.tokenizer().identifier_mode.set(false);
    }

    /// Returns true if the tokenizer is in identifier mode.
    fn is_identifier_mode(&self) -> bool {
        self.tokenizer().identifier_mode.get()
    }

    /// Returns the tokenizer to its default mode.
    fn exit_mode(&self) {
        if self.is_identifier_mode() {
            self.exit_identifier_mode();
        }
    }

    /// Advance the tokenizer by one character.
    ///
    /// If the input has been exhausted, then this returns `None`.
    fn scan(&self) -> Option<char> {
        if self.is_eof() {
            return None;
        }
        let Position {
            mut offset,
            mut line,
            mut column,
        } = self.pos();

        if self.char() == '\n' {
            line = line.checked_add(1).unwrap();
            column = 1;
        } else {
            column = column.checked_add(1).unwrap();
        }

        offset += self.char().len_utf8();
        self.tokenizer().pos.set(Position {
            offset,
            line,
            column,
        });
        self.text()[self.offset()..].chars().next()
    }

    /// Tokenize the text.
    pub(crate) fn tokenize(&self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        self.tokenizer().reset();

        loop {
            if self.is_eof() {
                break;
            }

            match self.char() {
                '─' | '│' if self.is_identifier_mode() => {
                    self.error(self.span(), ErrorKind::IdentifierCharInvalid(self.char()));
                }
                ' ' | '─' | '│' => {}
                '\n' | '\t' | '\r' => {
                    self.exit_mode();
                }
                '├' => tokens.push(Token {
                    kind: TokenKind::Tee,
                    span: self.span(),
                    lexeme: "├".to_string(),
                }),
                '└' => tokens.push(Token {
                    kind: TokenKind::Corner,
                    span: self.span(),
                    lexeme: "└".to_string(),
                }),
                // Comments start with `//`.
                '/' if self.peek().is_some_and(|c| c == '/') => {
                    self.exit_mode();
                    self.scan_comments();
                }
                _ => {
                    let token = self.scan_word()?;
                    if token.kind == TokenKind::When || token.kind == TokenKind::Given {
                        self.enter_identifier_mode()
                    }
                    tokens.push(token);
                }
            }

            if self.scan().is_none() {
                break;
            }
        }

        Ok(tokens)
    }

    /// Discards all characters until the end of the line.
    fn scan_comments(&self) {
        loop {
            match self.peek() {
                Some('\n') => break,
                Some(_) => {
                    self.scan();
                }
                None => break,
            }
        }
    }

    /// Consumes a word from the input.
    ///
    /// A word is defined as a sequence of characters that are not whitespace.
    /// If the word is a keyword, then the appropriate token is returned.
    /// Otherwise, a Word token is returned.
    fn scan_word(&self) -> Result<Token> {
        let mut lexeme = String::new();
        let span_start = self.pos();

        loop {
            if self.is_identifier_mode() && !is_valid_identifier_char(self.char()) {
                return Err(self.error(self.span(), ErrorKind::IdentifierCharInvalid(self.char())));
            } else if self.peek().is_none() || self.peek().is_some_and(|c| c.is_whitespace()) {
                lexeme.push(self.char());
                let kind = match lexeme.to_lowercase().as_str() {
                    "when" => TokenKind::When,
                    "it" => TokenKind::It,
                    "given" => TokenKind::Given,
                    _ => TokenKind::Word,
                };
                return Ok(Token {
                    kind,
                    span: self.span().with_start(span_start),
                    lexeme,
                });
            } else {
                lexeme.push(self.char());
                self.scan();
            }
        }
    }
}

/// Checks whether a character might appear in an identifier.
///
/// Valid identifiers are those which can be used as a variable name
/// and `-`, which will be converted to `_` in the generated code.
fn is_valid_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '\'' || c == '"'
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::error::Result;
    use crate::span::{Position, Span};
    use crate::syntax::tokenizer::{
        self, ErrorKind::IdentifierCharInvalid, Token, TokenKind, Tokenizer,
    };

    #[derive(Clone, Debug)]
    struct TestError {
        span: Span,
        kind: tokenizer::ErrorKind,
    }

    impl PartialEq<tokenizer::Error> for TestError {
        fn eq(&self, other: &tokenizer::Error) -> bool {
            self.span == other.span && self.kind == other.kind
        }
    }

    impl PartialEq<TestError> for tokenizer::Error {
        fn eq(&self, other: &TestError) -> bool {
            self.span == other.span && self.kind == other.kind
        }
    }

    fn e(kind: tokenizer::ErrorKind, span: Span) -> TestError {
        TestError { kind, span }
    }

    fn p(offset: usize, line: usize, column: usize) -> Position {
        Position::new(offset, line, column)
    }

    fn s(start: Position, end: Position) -> Span {
        Span::new(start, end)
    }

    fn t(kind: TokenKind, lexeme: &str, span: Span) -> Token {
        Token {
            kind,
            lexeme: lexeme.to_string(),
            span,
        }
    }

    fn tokenize(text: &str) -> tokenizer::Result<Vec<Token>> {
        Tokenizer::new().tokenize(text)
    }

    #[test]
    fn test_only_contract_name() -> Result<()> {
        let simple_name = String::from("Foo");
        let starts_whitespace = String::from(" Foo");
        let ends_whitespace = String::from("Foo ");

        let mut tokenizer = Tokenizer::new();

        assert_eq!(
            tokenizer.tokenize(&simple_name)?,
            vec![t(TokenKind::Word, "Foo", s(p(0, 1, 1), p(2, 1, 3)))]
        );
        assert_eq!(
            tokenizer.tokenize(&starts_whitespace)?,
            vec![t(TokenKind::Word, "Foo", s(p(1, 1, 2), p(3, 1, 4)))]
        );
        assert_eq!(
            tokenizer.tokenize(&ends_whitespace)?,
            vec![t(TokenKind::Word, "Foo", s(p(0, 1, 1), p(2, 1, 3)))]
        );

        Ok(())
    }

    #[test]
    fn test_comments() -> Result<()> {
        let file_contents = String::from(
            "Foo_Test\n└── when something bad happens // some comments \n   └── it should revert",
        );

        assert_eq!(
            tokenize(&file_contents)?,
            vec![
                t(TokenKind::Word, "Foo_Test", s(p(0, 1, 1), p(7, 1, 8))),
                t(TokenKind::Corner, "└", s(p(9, 2, 1), p(9, 2, 1))),
                t(TokenKind::When, "when", s(p(19, 2, 5), p(22, 2, 8))),
                t(TokenKind::Word, "something", s(p(24, 2, 10), p(32, 2, 18))),
                t(TokenKind::Word, "bad", s(p(34, 2, 20), p(36, 2, 22))),
                t(TokenKind::Word, "happens", s(p(38, 2, 24), p(44, 2, 30))),
                t(TokenKind::Corner, "└", s(p(67, 3, 4), p(67, 3, 4))),
                t(TokenKind::It, "it", s(p(77, 3, 8), p(78, 3, 9))),
                t(TokenKind::Word, "should", s(p(80, 3, 11), p(85, 3, 16))),
                t(TokenKind::Word, "revert", s(p(87, 3, 18), p(92, 3, 23))),
            ]
        );

        let file_contents = String::from(
            "Foo_Test\n└── when something bad happens\n // some comments \n   └── it should revert",
        );

        assert_eq!(
            tokenize(&file_contents)?,
            vec![
                t(TokenKind::Word, "Foo_Test", s(p(0, 1, 1), p(7, 1, 8))),
                t(TokenKind::Corner, "└", s(p(9, 2, 1), p(9, 2, 1))),
                t(TokenKind::When, "when", s(p(19, 2, 5), p(22, 2, 8))),
                t(TokenKind::Word, "something", s(p(24, 2, 10), p(32, 2, 18))),
                t(TokenKind::Word, "bad", s(p(34, 2, 20), p(36, 2, 22))),
                t(TokenKind::Word, "happens", s(p(38, 2, 24), p(44, 2, 30))),
                t(TokenKind::Corner, "└", s(p(68, 4, 4), p(68, 4, 4))),
                t(TokenKind::It, "it", s(p(78, 4, 8), p(79, 4, 9))),
                t(TokenKind::Word, "should", s(p(81, 4, 11), p(86, 4, 16))),
                t(TokenKind::Word, "revert", s(p(88, 4, 18), p(93, 4, 23))),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_invalid_characters() {
        assert_eq!(
            tokenize("foo\n└── when |weird identifier").unwrap_err(),
            e(IdentifierCharInvalid('|'), s(p(19, 2, 10), p(19, 2, 10)),)
        );
        assert_eq!(
            tokenize("foo\n└── when w|eird identifier").unwrap_err(),
            e(IdentifierCharInvalid('|'), s(p(20, 2, 11), p(20, 2, 11)),)
        );
        assert_eq!(
            tokenize("foo\n└── when weird| identifier").unwrap_err(),
            e(IdentifierCharInvalid('|'), s(p(24, 2, 15), p(24, 2, 15)),)
        );
        assert_eq!(
            tokenize("foo\n└── when .weird identifier").unwrap_err(),
            e(IdentifierCharInvalid('.'), s(p(19, 2, 10), p(19, 2, 10)),)
        );
        assert_eq!(
            tokenize("foo\n└── when w,eird identifier").unwrap_err(),
            e(IdentifierCharInvalid(','), s(p(20, 2, 11), p(20, 2, 11)),)
        );
        assert_eq!(
            tokenize("foo\n└── given |weird identifier").unwrap_err(),
            e(IdentifierCharInvalid('|'), s(p(20, 2, 11), p(20, 2, 11)),)
        );
        assert_eq!(
            tokenize("foo\n└── given w|eird identifier").unwrap_err(),
            e(IdentifierCharInvalid('|'), s(p(21, 2, 12), p(21, 2, 12)),)
        );
        assert_eq!(
            tokenize("foo\n└── given weird| identifier").unwrap_err(),
            e(IdentifierCharInvalid('|'), s(p(25, 2, 16), p(25, 2, 16)),)
        );
        assert_eq!(
            tokenize("foo\n└── given .weird identifier").unwrap_err(),
            e(IdentifierCharInvalid('.'), s(p(20, 2, 11), p(20, 2, 11)),)
        );
        assert_eq!(
            tokenize("foo\n└── given w,eird identifier").unwrap_err(),
            e(IdentifierCharInvalid(','), s(p(21, 2, 12), p(21, 2, 12)),)
        );
    }

    #[test]
    fn test_only_filename_and_newline() {
        let simple_name = String::from("foo\n");
        let starts_whitespace = String::from(" foo\n");
        let ends_whitespace = String::from("foo \n");

        let expected = vec![t(TokenKind::Word, "foo", s(p(0, 1, 1), p(2, 1, 3)))];
        let mut tokenizer = Tokenizer::new();

        assert_eq!(tokenizer.tokenize(&simple_name).unwrap(), expected);
        assert_eq!(
            tokenizer.tokenize(&starts_whitespace).unwrap(),
            vec![t(TokenKind::Word, "foo", s(p(1, 1, 2), p(3, 1, 4)))]
        );
        assert_eq!(tokenizer.tokenize(&ends_whitespace).unwrap(), expected);
    }

    #[test]
    fn test_one_child() {
        // Test parsing a when.
        let file_contents =
            String::from("Foo_Test\n└── when something bad happens\n   └── it should revert");

        assert_eq!(
            tokenize(&file_contents).unwrap(),
            vec![
                t(TokenKind::Word, "Foo_Test", s(p(0, 1, 1), p(7, 1, 8))),
                t(TokenKind::Corner, "└", s(p(9, 2, 1), p(9, 2, 1))),
                t(TokenKind::When, "when", s(p(19, 2, 5), p(22, 2, 8))),
                t(TokenKind::Word, "something", s(p(24, 2, 10), p(32, 2, 18))),
                t(TokenKind::Word, "bad", s(p(34, 2, 20), p(36, 2, 22))),
                t(TokenKind::Word, "happens", s(p(38, 2, 24), p(44, 2, 30))),
                t(TokenKind::Corner, "└", s(p(49, 3, 4), p(49, 3, 4))),
                t(TokenKind::It, "it", s(p(59, 3, 8), p(60, 3, 9))),
                t(TokenKind::Word, "should", s(p(62, 3, 11), p(67, 3, 16))),
                t(TokenKind::Word, "revert", s(p(69, 3, 18), p(74, 3, 23))),
            ]
        );

        // Test parsing a given.
        let file_contents =
            String::from("Foo_Test\n└── given something bad happens\n   └── it should revert");

        assert_eq!(
            tokenize(&file_contents).unwrap(),
            vec![
                t(TokenKind::Word, "Foo_Test", s(p(0, 1, 1), p(7, 1, 8))),
                t(TokenKind::Corner, "└", s(p(9, 2, 1), p(9, 2, 1))),
                t(TokenKind::Given, "given", s(p(19, 2, 5), p(23, 2, 9))),
                t(TokenKind::Word, "something", s(p(25, 2, 11), p(33, 2, 19))),
                t(TokenKind::Word, "bad", s(p(35, 2, 21), p(37, 2, 23))),
                t(TokenKind::Word, "happens", s(p(39, 2, 25), p(45, 2, 31))),
                t(TokenKind::Corner, "└", s(p(50, 3, 4), p(50, 3, 4))),
                t(TokenKind::It, "it", s(p(60, 3, 8), p(61, 3, 9))),
                t(TokenKind::Word, "should", s(p(63, 3, 11), p(68, 3, 16))),
                t(TokenKind::Word, "revert", s(p(70, 3, 18), p(75, 3, 23))),
            ]
        );
    }

    #[test]
    fn test_multiple_children() {
        let file_contents = String::from(
            r#"multiple_children.t.sol
├── when stuff called
│  └── it should revert
└── when not stuff called
   ├── when the deposit amount is zero
   │  └── it should revert
   └── when the deposit amount is not zero
      ├── when the number count is zero
      │  └── it should revert
      ├── when the asset is not a contract
      │  └── it should revert
      └── when the asset is a contract
          ├── when the asset misses the ERC-20 return value
          │  ├── it should create the child
          │  ├── it should perform the ERC-20 transfers
          │  └── it should emit a {MultipleChildren} event
          └── when the asset does not miss the ERC-20 return value
              ├── it should create the child
              └── it should emit a {MultipleChildren} event"#,
        );

        let tokens = tokenize(&file_contents).unwrap();
        let expected = vec![
            t(
                TokenKind::Word,
                "multiple_children.t.sol",
                s(p(0, 1, 1), p(22, 1, 23)),
            ),
            t(TokenKind::Tee, "├", s(p(24, 2, 1), p(24, 2, 1))),
            t(TokenKind::When, "when", s(p(34, 2, 5), p(37, 2, 8))),
            t(TokenKind::Word, "stuff", s(p(39, 2, 10), p(43, 2, 14))),
            t(TokenKind::Word, "called", s(p(45, 2, 16), p(50, 2, 21))),
            t(TokenKind::Corner, "└", s(p(57, 3, 4), p(57, 3, 4))),
            t(TokenKind::It, "it", s(p(67, 3, 8), p(68, 3, 9))),
            t(TokenKind::Word, "should", s(p(70, 3, 11), p(75, 3, 16))),
            t(TokenKind::Word, "revert", s(p(77, 3, 18), p(82, 3, 23))),
            t(TokenKind::Corner, "└", s(p(84, 4, 1), p(84, 4, 1))),
            t(TokenKind::When, "when", s(p(94, 4, 5), p(97, 4, 8))),
            t(TokenKind::Word, "not", s(p(99, 4, 10), p(101, 4, 12))),
            t(TokenKind::Word, "stuff", s(p(103, 4, 14), p(107, 4, 18))),
            t(TokenKind::Word, "called", s(p(109, 4, 20), p(114, 4, 25))),
            t(TokenKind::Tee, "├", s(p(119, 5, 4), p(119, 5, 4))),
            t(TokenKind::When, "when", s(p(129, 5, 8), p(132, 5, 11))),
            t(TokenKind::Word, "the", s(p(134, 5, 13), p(136, 5, 15))),
            t(TokenKind::Word, "deposit", s(p(138, 5, 17), p(144, 5, 23))),
            t(TokenKind::Word, "amount", s(p(146, 5, 25), p(151, 5, 30))),
            t(TokenKind::Word, "is", s(p(153, 5, 32), p(154, 5, 33))),
            t(TokenKind::Word, "zero", s(p(156, 5, 35), p(159, 5, 38))),
            t(TokenKind::Corner, "└", s(p(169, 6, 7), p(169, 6, 7))),
            t(TokenKind::It, "it", s(p(179, 6, 11), p(180, 6, 12))),
            t(TokenKind::Word, "should", s(p(182, 6, 14), p(187, 6, 19))),
            t(TokenKind::Word, "revert", s(p(189, 6, 21), p(194, 6, 26))),
            t(TokenKind::Corner, "└", s(p(199, 7, 4), p(199, 7, 4))),
            t(TokenKind::When, "when", s(p(209, 7, 8), p(212, 7, 11))),
            t(TokenKind::Word, "the", s(p(214, 7, 13), p(216, 7, 15))),
            t(TokenKind::Word, "deposit", s(p(218, 7, 17), p(224, 7, 23))),
            t(TokenKind::Word, "amount", s(p(226, 7, 25), p(231, 7, 30))),
            t(TokenKind::Word, "is", s(p(233, 7, 32), p(234, 7, 33))),
            t(TokenKind::Word, "not", s(p(236, 7, 35), p(238, 7, 37))),
            t(TokenKind::Word, "zero", s(p(240, 7, 39), p(243, 7, 42))),
            t(TokenKind::Tee, "├", s(p(251, 8, 7), p(251, 8, 7))),
            t(TokenKind::When, "when", s(p(261, 8, 11), p(264, 8, 14))),
            t(TokenKind::Word, "the", s(p(266, 8, 16), p(268, 8, 18))),
            t(TokenKind::Word, "number", s(p(270, 8, 20), p(275, 8, 25))),
            t(TokenKind::Word, "count", s(p(277, 8, 27), p(281, 8, 31))),
            t(TokenKind::Word, "is", s(p(283, 8, 33), p(284, 8, 34))),
            t(TokenKind::Word, "zero", s(p(286, 8, 36), p(289, 8, 39))),
            t(TokenKind::Corner, "└", s(p(302, 9, 10), p(302, 9, 10))),
            t(TokenKind::It, "it", s(p(312, 9, 14), p(313, 9, 15))),
            t(TokenKind::Word, "should", s(p(315, 9, 17), p(320, 9, 22))),
            t(TokenKind::Word, "revert", s(p(322, 9, 24), p(327, 9, 29))),
            t(TokenKind::Tee, "├", s(p(335, 10, 7), p(335, 10, 7))),
            t(TokenKind::When, "when", s(p(345, 10, 11), p(348, 10, 14))),
            t(TokenKind::Word, "the", s(p(350, 10, 16), p(352, 10, 18))),
            t(TokenKind::Word, "asset", s(p(354, 10, 20), p(358, 10, 24))),
            t(TokenKind::Word, "is", s(p(360, 10, 26), p(361, 10, 27))),
            t(TokenKind::Word, "not", s(p(363, 10, 29), p(365, 10, 31))),
            t(TokenKind::Word, "a", s(p(367, 10, 33), p(367, 10, 33))),
            t(
                TokenKind::Word,
                "contract",
                s(p(369, 10, 35), p(376, 10, 42)),
            ),
            t(TokenKind::Corner, "└", s(p(389, 11, 10), p(389, 11, 10))),
            t(TokenKind::It, "it", s(p(399, 11, 14), p(400, 11, 15))),
            t(TokenKind::Word, "should", s(p(402, 11, 17), p(407, 11, 22))),
            t(TokenKind::Word, "revert", s(p(409, 11, 24), p(414, 11, 29))),
            t(TokenKind::Corner, "└", s(p(422, 12, 7), p(422, 12, 7))),
            t(TokenKind::When, "when", s(p(432, 12, 11), p(435, 12, 14))),
            t(TokenKind::Word, "the", s(p(437, 12, 16), p(439, 12, 18))),
            t(TokenKind::Word, "asset", s(p(441, 12, 20), p(445, 12, 24))),
            t(TokenKind::Word, "is", s(p(447, 12, 26), p(448, 12, 27))),
            t(TokenKind::Word, "a", s(p(450, 12, 29), p(450, 12, 29))),
            t(
                TokenKind::Word,
                "contract",
                s(p(452, 12, 31), p(459, 12, 38)),
            ),
            t(TokenKind::Tee, "├", s(p(471, 13, 11), p(471, 13, 11))),
            t(TokenKind::When, "when", s(p(481, 13, 15), p(484, 13, 18))),
            t(TokenKind::Word, "the", s(p(486, 13, 20), p(488, 13, 22))),
            t(TokenKind::Word, "asset", s(p(490, 13, 24), p(494, 13, 28))),
            t(TokenKind::Word, "misses", s(p(496, 13, 30), p(501, 13, 35))),
            t(TokenKind::Word, "the", s(p(503, 13, 37), p(505, 13, 39))),
            t(TokenKind::Word, "ERC-20", s(p(507, 13, 41), p(512, 13, 46))),
            t(TokenKind::Word, "return", s(p(514, 13, 48), p(519, 13, 53))),
            t(TokenKind::Word, "value", s(p(521, 13, 55), p(525, 13, 59))),
            t(TokenKind::Tee, "├", s(p(542, 14, 14), p(542, 14, 14))),
            t(TokenKind::It, "it", s(p(552, 14, 18), p(553, 14, 19))),
            t(TokenKind::Word, "should", s(p(555, 14, 21), p(560, 14, 26))),
            t(TokenKind::Word, "create", s(p(562, 14, 28), p(567, 14, 33))),
            t(TokenKind::Word, "the", s(p(569, 14, 35), p(571, 14, 37))),
            t(TokenKind::Word, "child", s(p(573, 14, 39), p(577, 14, 43))),
            t(TokenKind::Tee, "├", s(p(594, 15, 14), p(594, 15, 14))),
            t(TokenKind::It, "it", s(p(604, 15, 18), p(605, 15, 19))),
            t(TokenKind::Word, "should", s(p(607, 15, 21), p(612, 15, 26))),
            t(
                TokenKind::Word,
                "perform",
                s(p(614, 15, 28), p(620, 15, 34)),
            ),
            t(TokenKind::Word, "the", s(p(622, 15, 36), p(624, 15, 38))),
            t(TokenKind::Word, "ERC-20", s(p(626, 15, 40), p(631, 15, 45))),
            t(
                TokenKind::Word,
                "transfers",
                s(p(633, 15, 47), p(641, 15, 55)),
            ),
            t(TokenKind::Corner, "└", s(p(658, 16, 14), p(658, 16, 14))),
            t(TokenKind::It, "it", s(p(668, 16, 18), p(669, 16, 19))),
            t(TokenKind::Word, "should", s(p(671, 16, 21), p(676, 16, 26))),
            t(TokenKind::Word, "emit", s(p(678, 16, 28), p(681, 16, 31))),
            t(TokenKind::Word, "a", s(p(683, 16, 33), p(683, 16, 33))),
            t(
                TokenKind::Word,
                "{MultipleChildren}",
                s(p(685, 16, 35), p(702, 16, 52)),
            ),
            t(TokenKind::Word, "event", s(p(704, 16, 54), p(708, 16, 58))),
            t(TokenKind::Corner, "└", s(p(720, 17, 11), p(720, 17, 11))),
            t(TokenKind::When, "when", s(p(730, 17, 15), p(733, 17, 18))),
            t(TokenKind::Word, "the", s(p(735, 17, 20), p(737, 17, 22))),
            t(TokenKind::Word, "asset", s(p(739, 17, 24), p(743, 17, 28))),
            t(TokenKind::Word, "does", s(p(745, 17, 30), p(748, 17, 33))),
            t(TokenKind::Word, "not", s(p(750, 17, 35), p(752, 17, 37))),
            t(TokenKind::Word, "miss", s(p(754, 17, 39), p(757, 17, 42))),
            t(TokenKind::Word, "the", s(p(759, 17, 44), p(761, 17, 46))),
            t(TokenKind::Word, "ERC-20", s(p(763, 17, 48), p(768, 17, 53))),
            t(TokenKind::Word, "return", s(p(770, 17, 55), p(775, 17, 60))),
            t(TokenKind::Word, "value", s(p(777, 17, 62), p(781, 17, 66))),
            t(TokenKind::Tee, "├", s(p(797, 18, 15), p(797, 18, 15))),
            t(TokenKind::It, "it", s(p(807, 18, 19), p(808, 18, 20))),
            t(TokenKind::Word, "should", s(p(810, 18, 22), p(815, 18, 27))),
            t(TokenKind::Word, "create", s(p(817, 18, 29), p(822, 18, 34))),
            t(TokenKind::Word, "the", s(p(824, 18, 36), p(826, 18, 38))),
            t(TokenKind::Word, "child", s(p(828, 18, 40), p(832, 18, 44))),
            t(TokenKind::Corner, "└", s(p(848, 19, 15), p(848, 19, 15))),
            t(TokenKind::It, "it", s(p(858, 19, 19), p(859, 19, 20))),
            t(TokenKind::Word, "should", s(p(861, 19, 22), p(866, 19, 27))),
            t(TokenKind::Word, "emit", s(p(868, 19, 29), p(871, 19, 32))),
            t(TokenKind::Word, "a", s(p(873, 19, 34), p(873, 19, 34))),
            t(
                TokenKind::Word,
                "{MultipleChildren}",
                s(p(875, 19, 36), p(892, 19, 53)),
            ),
            t(TokenKind::Word, "event", s(p(894, 19, 55), p(898, 19, 59))),
        ];

        assert_eq!(tokens.len(), expected.len());
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_case_insensitive_keywords() {
        let file_contents =
            String::from("Foo_Test\n└── GIVEN something bad happens\n   └── whEN stuff is true\n   └── It should revert.");

        assert_eq!(
            tokenize(&file_contents).unwrap(),
            vec![
                t(TokenKind::Word, "Foo_Test", s(p(0, 1, 1), p(7, 1, 8))),
                t(TokenKind::Corner, "└", s(p(9, 2, 1), p(9, 2, 1))),
                t(TokenKind::Given, "GIVEN", s(p(19, 2, 5), p(23, 2, 9))),
                t(TokenKind::Word, "something", s(p(25, 2, 11), p(33, 2, 19))),
                t(TokenKind::Word, "bad", s(p(35, 2, 21), p(37, 2, 23))),
                t(TokenKind::Word, "happens", s(p(39, 2, 25), p(45, 2, 31))),
                t(TokenKind::Corner, "└", s(p(50, 3, 4), p(50, 3, 4))),
                t(TokenKind::When, "whEN", s(p(60, 3, 8), p(63, 3, 11))),
                t(TokenKind::Word, "stuff", s(p(65, 3, 13), p(69, 3, 17))),
                t(TokenKind::Word, "is", s(p(71, 3, 19), p(72, 3, 20))),
                t(TokenKind::Word, "true", s(p(74, 3, 22), p(77, 3, 25))),
                t(TokenKind::Corner, "└", s(p(82, 4, 4), p(82, 4, 4))),
                t(TokenKind::It, "It", s(p(92, 4, 8), p(93, 4, 9))),
                t(TokenKind::Word, "should", s(p(95, 4, 11), p(100, 4, 16))),
                t(TokenKind::Word, "revert.", s(p(102, 4, 18), p(108, 4, 24))),
            ]
        );
    }
}
