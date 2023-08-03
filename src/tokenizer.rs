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
    /// Found an invalid filename character.
    FileNameCharInvalid(char),
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
            FileNameCharInvalid(c) => write!(f, "invalid filename: {:?}", c),
            IdentifierCharInvalid(c) => write!(f, "invalid identifier: {:?}", c),
            _ => unreachable!(),
        }
    }
}

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

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    TEE,
    CORNER,
    STRING,
    WHEN,
    IT,
}

type Tokens = Vec<Token>;

pub struct Tokenizer {
    pos: Cell<Position>,
    /// When true, the tokenizer is in `identifier` mode.
    ///
    /// In `identifier` mode, the tokenizer will error if it encounters a
    /// a character that is not a valid identifier character.
    /// This is to prevent malformed names when emitting identifiers.
    identifier_mode: Cell<bool>,
    /// When true, the tokenizer is in `filename` mode.
    ///
    /// In `filename` mode, the tokenizer will error if it encounters a
    /// a character that is not a valid filename character.
    /// This is to prevent malformed names when creating the output file.
    filename_mode: Cell<bool>,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            pos: Cell::new(Position::new(0, 1, 1)),
            identifier_mode: Cell::new(false),
            // Starts as `true` because the first token must always be a filename.
            filename_mode: Cell::new(true),
        }
    }

    /// Tokenize the tree.
    pub fn tokenize(&mut self, text: &str) -> Result<Tokens> {
        TokenizerI::new(self, text).tokenize()
    }

    /// Reset the tokenizer's state.
    fn reset(&self) {
        self.pos.set(Position::new(0, 1, 1));
        self.identifier_mode.set(false);
        self.filename_mode.set(true);
    }
}

struct TokenizerI<'s, T> {
    text: &'s str,
    tokenizer: T,
}

impl<'s, T: Borrow<Tokenizer>> TokenizerI<'s, T> {
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

    /// Return the current line number of the tokenizer.
    ///
    /// The line number starts at `1`.
    fn line(&self) -> usize {
        self.tokenizer().pos.get().line
    }

    /// Return the current column of the tokenizer.
    ///
    /// The column number starts at `1` and is reset whenever a `\n` is seen.
    fn column(&self) -> usize {
        self.tokenizer().pos.get().column
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

    /// Exits filename mode.
    fn exit_filename_mode(&self) {
        self.tokenizer().filename_mode.set(false);
    }

    /// Returns true if the tokenizer is in filename mode.
    fn is_filename_mode(&self) -> bool {
        self.tokenizer().filename_mode.get()
    }

    /// Returns the tokenizer to its default mode.
    fn exit_mode(&self) {
        if self.is_filename_mode() {
            self.exit_filename_mode();
        }
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
    pub fn tokenize(&self) -> Result<Tokens> {
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
                '─' | '│' if self.is_filename_mode() => {
                    self.error(self.span(), ErrorKind::FileNameCharInvalid(self.char()));
                }
                ' ' | '─' | '│' => {}
                '\n' | '\t' | '\r' => {
                    self.exit_mode();
                }
                '├' => tokens.push(Token {
                    kind: TokenKind::TEE,
                    span: self.span(),
                    lexeme: "├".to_string(),
                }),
                '└' => tokens.push(Token {
                    kind: TokenKind::CORNER,
                    span: self.span(),
                    lexeme: "└".to_string(),
                }),
                '/' if self.peek().is_some_and(|c| c == '/') => {
                    self.exit_mode();
                    self.scan_comments();
                }
                _ => {
                    let token = self.scan_word()?;
                    match token.kind {
                        TokenKind::WHEN => self.enter_identifier_mode(),
                        _ => {}
                    }
                    tokens.push(token);
                }
            }

            if let None = self.scan() {
                break;
            }
        }

        Ok(tokens)
    }

    /// Parse a horizontal line.
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

    fn scan_word(&self) -> Result<Token> {
        let mut lexeme = String::new();
        let span_start = self.pos();

        loop {
            if self.is_identifier_mode() && !is_valid_identifier_char(self.char()) {
                return Err(self
                    .error(self.span(), ErrorKind::IdentifierCharInvalid(self.char()))
                    .into());
            } else if self.is_filename_mode() && !is_valid_filename_char(self.char()) {
                return Err(self
                    .error(self.span(), ErrorKind::FileNameCharInvalid(self.char()))
                    .into());
            } else if self.peek().is_none() || self.peek().is_some_and(|c| c.is_whitespace()) {
                lexeme.push(self.char());
                let kind = match lexeme.as_str() {
                    "when" => TokenKind::WHEN,
                    "it" => TokenKind::IT,
                    _ => TokenKind::STRING,
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
/// Valid identifiers are those which can be used as a variable name.
fn is_valid_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Checks whether a character might appear in a filename.
fn is_valid_filename_char(c: char) -> bool {
    is_valid_identifier_char(c) || c == '.'
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::error::Result;
    use crate::{
        span::{Position, Span},
        tokenizer::{self, Token, TokenKind, Tokenizer},
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

    #[test]
    fn test_only_filename() -> Result<()> {
        let simple_name = String::from("foo");
        let starts_whitespace = String::from(" foo");
        let ends_whitespace = String::from("foo ");

        let mut tokenizer = Tokenizer::new();

        assert_eq!(
            tokenizer.tokenize(&simple_name)?,
            vec![t(TokenKind::STRING, "foo", s(p(0, 1, 1), p(2, 1, 3)))]
        );
        assert_eq!(
            tokenizer.tokenize(&starts_whitespace)?,
            vec![t(TokenKind::STRING, "foo", s(p(1, 1, 2), p(3, 1, 4)))]
        );
        assert_eq!(
            tokenizer.tokenize(&ends_whitespace)?,
            vec![t(TokenKind::STRING, "foo", s(p(0, 1, 1), p(2, 1, 3)))]
        );

        Ok(())
    }

    #[test]
    fn test_invalid_characters() {
        assert_eq!(
            Tokenizer::new().tokenize("/foobar").unwrap_err(),
            TestError {
                span: s(p(0, 1, 1), p(0, 1, 1)),
                kind: tokenizer::ErrorKind::FileNameCharInvalid('/'),
            }
        );
        assert_eq!(
            Tokenizer::new().tokenize("foo/bar").unwrap_err(),
            TestError {
                span: s(p(3, 1, 4), p(3, 1, 4)),
                kind: tokenizer::ErrorKind::FileNameCharInvalid('/'),
            }
        );
        assert_eq!(
            Tokenizer::new().tokenize("foobar/").unwrap_err(),
            TestError {
                span: s(p(6, 1, 7), p(6, 1, 7)),
                kind: tokenizer::ErrorKind::FileNameCharInvalid('/'),
            }
        );
        assert_eq!(
            Tokenizer::new()
                .tokenize("foo\n└── when |weird identifier")
                .unwrap_err(),
            TestError {
                span: s(p(19, 2, 10), p(19, 2, 10)),
                kind: tokenizer::ErrorKind::IdentifierCharInvalid('|'),
            }
        );
        assert_eq!(
            Tokenizer::new()
                .tokenize("foo\n└── when w|eird identifier")
                .unwrap_err(),
            TestError {
                span: s(p(20, 2, 11), p(20, 2, 11)),
                kind: tokenizer::ErrorKind::IdentifierCharInvalid('|'),
            }
        );
        assert_eq!(
            Tokenizer::new()
                .tokenize("foo\n└── when weird| identifier")
                .unwrap_err(),
            TestError {
                span: s(p(24, 2, 15), p(24, 2, 15)),
                kind: tokenizer::ErrorKind::IdentifierCharInvalid('|'),
            }
        );
        assert_eq!(
            Tokenizer::new()
                .tokenize("foo\n└── when .weird identifier")
                .unwrap_err(),
            TestError {
                span: s(p(19, 2, 10), p(19, 2, 10)),
                kind: tokenizer::ErrorKind::IdentifierCharInvalid('.'),
            }
        );
        assert_eq!(
            Tokenizer::new()
                .tokenize("foo\n└── when w,eird identifier")
                .unwrap_err(),
            TestError {
                span: s(p(20, 2, 11), p(20, 2, 11)),
                kind: tokenizer::ErrorKind::IdentifierCharInvalid(','),
            }
        );
        assert_eq!(
            Tokenizer::new()
                .tokenize("foo\n└── when weird' identifier")
                .unwrap_err(),
            TestError {
                span: s(p(24, 2, 15), p(24, 2, 15)),
                kind: tokenizer::ErrorKind::IdentifierCharInvalid('\''),
            }
        );
    }

    #[test]
    fn test_only_filename_and_newline() -> Result<()> {
        let simple_name = String::from("foo\n");
        let starts_whitespace = String::from(" foo\n");
        let ends_whitespace = String::from("foo \n");

        let expected = vec![t(TokenKind::STRING, "foo", s(p(0, 1, 1), p(2, 1, 3)))];
        let mut tokenizer = Tokenizer::new();

        assert_eq!(tokenizer.tokenize(&simple_name)?, expected);
        assert_eq!(
            tokenizer.tokenize(&starts_whitespace)?,
            vec![t(TokenKind::STRING, "foo", s(p(1, 1, 2), p(3, 1, 4)))]
        );
        assert_eq!(tokenizer.tokenize(&ends_whitespace)?, expected);

        Ok(())
    }

    #[test]
    fn test_one_child() -> Result<()> {
        let file_contents =
            String::from("file.sol\n└── when something bad happens\n   └── it should revert");

        assert_eq!(
            Tokenizer::new().tokenize(&file_contents)?,
            vec![
                t(TokenKind::STRING, "file.sol", s(p(0, 1, 1), p(7, 1, 8))),
                t(TokenKind::CORNER, "└", s(p(9, 2, 1), p(9, 2, 1))),
                t(TokenKind::WHEN, "when", s(p(19, 2, 5), p(22, 2, 8))),
                t(
                    TokenKind::STRING,
                    "something",
                    s(p(24, 2, 10), p(32, 2, 18))
                ),
                t(TokenKind::STRING, "bad", s(p(34, 2, 20), p(36, 2, 22))),
                t(TokenKind::STRING, "happens", s(p(38, 2, 24), p(44, 2, 30))),
                t(TokenKind::CORNER, "└", s(p(49, 3, 4), p(49, 3, 4))),
                t(TokenKind::IT, "it", s(p(59, 3, 8), p(60, 3, 9))),
                t(TokenKind::STRING, "should", s(p(62, 3, 11), p(67, 3, 16))),
                t(TokenKind::STRING, "revert", s(p(69, 3, 18), p(74, 3, 23))),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_multiple_children() -> Result<()> {
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
          ├── when the asset misses the ERC_20 return value
          │  ├── it should create the child
          │  ├── it should perform the ERC_20 transfers
          │  └── it should emit a {MultipleChildren} event
          └── when the asset does not miss the ERC_20 return value
              ├── it should create the child
              └── it should emit a {MultipleChildren} event"#,
        );

        let tokens = Tokenizer::new().tokenize(&file_contents)?;
        let expected = vec![
            t(
                TokenKind::STRING,
                "multiple_children.t.sol",
                s(p(0, 1, 1), p(22, 1, 23)),
            ),
            t(TokenKind::TEE, "├", s(p(24, 2, 1), p(24, 2, 1))),
            t(TokenKind::WHEN, "when", s(p(34, 2, 5), p(37, 2, 8))),
            t(TokenKind::STRING, "stuff", s(p(39, 2, 10), p(43, 2, 14))),
            t(TokenKind::STRING, "called", s(p(45, 2, 16), p(50, 2, 21))),
            t(TokenKind::CORNER, "└", s(p(57, 3, 4), p(57, 3, 4))),
            t(TokenKind::IT, "it", s(p(67, 3, 8), p(68, 3, 9))),
            t(TokenKind::STRING, "should", s(p(70, 3, 11), p(75, 3, 16))),
            t(TokenKind::STRING, "revert", s(p(77, 3, 18), p(82, 3, 23))),
            t(TokenKind::CORNER, "└", s(p(84, 4, 1), p(84, 4, 1))),
            t(TokenKind::WHEN, "when", s(p(94, 4, 5), p(97, 4, 8))),
            t(TokenKind::STRING, "not", s(p(99, 4, 10), p(101, 4, 12))),
            t(TokenKind::STRING, "stuff", s(p(103, 4, 14), p(107, 4, 18))),
            t(TokenKind::STRING, "called", s(p(109, 4, 20), p(114, 4, 25))),
            t(TokenKind::TEE, "├", s(p(119, 5, 4), p(119, 5, 4))),
            t(TokenKind::WHEN, "when", s(p(129, 5, 8), p(132, 5, 11))),
            t(TokenKind::STRING, "the", s(p(134, 5, 13), p(136, 5, 15))),
            t(
                TokenKind::STRING,
                "deposit",
                s(p(138, 5, 17), p(144, 5, 23)),
            ),
            t(TokenKind::STRING, "amount", s(p(146, 5, 25), p(151, 5, 30))),
            t(TokenKind::STRING, "is", s(p(153, 5, 32), p(154, 5, 33))),
            t(TokenKind::STRING, "zero", s(p(156, 5, 35), p(159, 5, 38))),
            t(TokenKind::CORNER, "└", s(p(169, 6, 7), p(169, 6, 7))),
            t(TokenKind::IT, "it", s(p(179, 6, 11), p(180, 6, 12))),
            t(TokenKind::STRING, "should", s(p(182, 6, 14), p(187, 6, 19))),
            t(TokenKind::STRING, "revert", s(p(189, 6, 21), p(194, 6, 26))),
            t(TokenKind::CORNER, "└", s(p(199, 7, 4), p(199, 7, 4))),
            t(TokenKind::WHEN, "when", s(p(209, 7, 8), p(212, 7, 11))),
            t(TokenKind::STRING, "the", s(p(214, 7, 13), p(216, 7, 15))),
            t(
                TokenKind::STRING,
                "deposit",
                s(p(218, 7, 17), p(224, 7, 23)),
            ),
            t(TokenKind::STRING, "amount", s(p(226, 7, 25), p(231, 7, 30))),
            t(TokenKind::STRING, "is", s(p(233, 7, 32), p(234, 7, 33))),
            t(TokenKind::STRING, "not", s(p(236, 7, 35), p(238, 7, 37))),
            t(TokenKind::STRING, "zero", s(p(240, 7, 39), p(243, 7, 42))),
            t(TokenKind::TEE, "├", s(p(251, 8, 7), p(251, 8, 7))),
            t(TokenKind::WHEN, "when", s(p(261, 8, 11), p(264, 8, 14))),
            t(TokenKind::STRING, "the", s(p(266, 8, 16), p(268, 8, 18))),
            t(TokenKind::STRING, "number", s(p(270, 8, 20), p(275, 8, 25))),
            t(TokenKind::STRING, "count", s(p(277, 8, 27), p(281, 8, 31))),
            t(TokenKind::STRING, "is", s(p(283, 8, 33), p(284, 8, 34))),
            t(TokenKind::STRING, "zero", s(p(286, 8, 36), p(289, 8, 39))),
            t(TokenKind::CORNER, "└", s(p(302, 9, 10), p(302, 9, 10))),
            t(TokenKind::IT, "it", s(p(312, 9, 14), p(313, 9, 15))),
            t(TokenKind::STRING, "should", s(p(315, 9, 17), p(320, 9, 22))),
            t(TokenKind::STRING, "revert", s(p(322, 9, 24), p(327, 9, 29))),
            t(TokenKind::TEE, "├", s(p(335, 10, 7), p(335, 10, 7))),
            t(TokenKind::WHEN, "when", s(p(345, 10, 11), p(348, 10, 14))),
            t(TokenKind::STRING, "the", s(p(350, 10, 16), p(352, 10, 18))),
            t(
                TokenKind::STRING,
                "asset",
                s(p(354, 10, 20), p(358, 10, 24)),
            ),
            t(TokenKind::STRING, "is", s(p(360, 10, 26), p(361, 10, 27))),
            t(TokenKind::STRING, "not", s(p(363, 10, 29), p(365, 10, 31))),
            t(TokenKind::STRING, "a", s(p(367, 10, 33), p(367, 10, 33))),
            t(
                TokenKind::STRING,
                "contract",
                s(p(369, 10, 35), p(376, 10, 42)),
            ),
            t(TokenKind::CORNER, "└", s(p(389, 11, 10), p(389, 11, 10))),
            t(TokenKind::IT, "it", s(p(399, 11, 14), p(400, 11, 15))),
            t(
                TokenKind::STRING,
                "should",
                s(p(402, 11, 17), p(407, 11, 22)),
            ),
            t(
                TokenKind::STRING,
                "revert",
                s(p(409, 11, 24), p(414, 11, 29)),
            ),
            t(TokenKind::CORNER, "└", s(p(422, 12, 7), p(422, 12, 7))),
            t(TokenKind::WHEN, "when", s(p(432, 12, 11), p(435, 12, 14))),
            t(TokenKind::STRING, "the", s(p(437, 12, 16), p(439, 12, 18))),
            t(
                TokenKind::STRING,
                "asset",
                s(p(441, 12, 20), p(445, 12, 24)),
            ),
            t(TokenKind::STRING, "is", s(p(447, 12, 26), p(448, 12, 27))),
            t(TokenKind::STRING, "a", s(p(450, 12, 29), p(450, 12, 29))),
            t(
                TokenKind::STRING,
                "contract",
                s(p(452, 12, 31), p(459, 12, 38)),
            ),
            t(TokenKind::TEE, "├", s(p(471, 13, 11), p(471, 13, 11))),
            t(TokenKind::WHEN, "when", s(p(481, 13, 15), p(484, 13, 18))),
            t(TokenKind::STRING, "the", s(p(486, 13, 20), p(488, 13, 22))),
            t(
                TokenKind::STRING,
                "asset",
                s(p(490, 13, 24), p(494, 13, 28)),
            ),
            t(
                TokenKind::STRING,
                "misses",
                s(p(496, 13, 30), p(501, 13, 35)),
            ),
            t(TokenKind::STRING, "the", s(p(503, 13, 37), p(505, 13, 39))),
            t(
                TokenKind::STRING,
                "ERC_20",
                s(p(507, 13, 41), p(512, 13, 46)),
            ),
            t(
                TokenKind::STRING,
                "return",
                s(p(514, 13, 48), p(519, 13, 53)),
            ),
            t(
                TokenKind::STRING,
                "value",
                s(p(521, 13, 55), p(525, 13, 59)),
            ),
            t(TokenKind::TEE, "├", s(p(542, 14, 14), p(542, 14, 14))),
            t(TokenKind::IT, "it", s(p(552, 14, 18), p(553, 14, 19))),
            t(
                TokenKind::STRING,
                "should",
                s(p(555, 14, 21), p(560, 14, 26)),
            ),
            t(
                TokenKind::STRING,
                "create",
                s(p(562, 14, 28), p(567, 14, 33)),
            ),
            t(TokenKind::STRING, "the", s(p(569, 14, 35), p(571, 14, 37))),
            t(
                TokenKind::STRING,
                "child",
                s(p(573, 14, 39), p(577, 14, 43)),
            ),
            t(TokenKind::TEE, "├", s(p(594, 15, 14), p(594, 15, 14))),
            t(TokenKind::IT, "it", s(p(604, 15, 18), p(605, 15, 19))),
            t(
                TokenKind::STRING,
                "should",
                s(p(607, 15, 21), p(612, 15, 26)),
            ),
            t(
                TokenKind::STRING,
                "perform",
                s(p(614, 15, 28), p(620, 15, 34)),
            ),
            t(TokenKind::STRING, "the", s(p(622, 15, 36), p(624, 15, 38))),
            t(
                TokenKind::STRING,
                "ERC_20",
                s(p(626, 15, 40), p(631, 15, 45)),
            ),
            t(
                TokenKind::STRING,
                "transfers",
                s(p(633, 15, 47), p(641, 15, 55)),
            ),
            t(TokenKind::CORNER, "└", s(p(658, 16, 14), p(658, 16, 14))),
            t(TokenKind::IT, "it", s(p(668, 16, 18), p(669, 16, 19))),
            t(
                TokenKind::STRING,
                "should",
                s(p(671, 16, 21), p(676, 16, 26)),
            ),
            t(TokenKind::STRING, "emit", s(p(678, 16, 28), p(681, 16, 31))),
            t(TokenKind::STRING, "a", s(p(683, 16, 33), p(683, 16, 33))),
            t(
                TokenKind::STRING,
                "{MultipleChildren}",
                s(p(685, 16, 35), p(702, 16, 52)),
            ),
            t(
                TokenKind::STRING,
                "event",
                s(p(704, 16, 54), p(708, 16, 58)),
            ),
            t(TokenKind::CORNER, "└", s(p(720, 17, 11), p(720, 17, 11))),
            t(TokenKind::WHEN, "when", s(p(730, 17, 15), p(733, 17, 18))),
            t(TokenKind::STRING, "the", s(p(735, 17, 20), p(737, 17, 22))),
            t(
                TokenKind::STRING,
                "asset",
                s(p(739, 17, 24), p(743, 17, 28)),
            ),
            t(TokenKind::STRING, "does", s(p(745, 17, 30), p(748, 17, 33))),
            t(TokenKind::STRING, "not", s(p(750, 17, 35), p(752, 17, 37))),
            t(TokenKind::STRING, "miss", s(p(754, 17, 39), p(757, 17, 42))),
            t(TokenKind::STRING, "the", s(p(759, 17, 44), p(761, 17, 46))),
            t(
                TokenKind::STRING,
                "ERC_20",
                s(p(763, 17, 48), p(768, 17, 53)),
            ),
            t(
                TokenKind::STRING,
                "return",
                s(p(770, 17, 55), p(775, 17, 60)),
            ),
            t(
                TokenKind::STRING,
                "value",
                s(p(777, 17, 62), p(781, 17, 66)),
            ),
            t(TokenKind::TEE, "├", s(p(797, 18, 15), p(797, 18, 15))),
            t(TokenKind::IT, "it", s(p(807, 18, 19), p(808, 18, 20))),
            t(
                TokenKind::STRING,
                "should",
                s(p(810, 18, 22), p(815, 18, 27)),
            ),
            t(
                TokenKind::STRING,
                "create",
                s(p(817, 18, 29), p(822, 18, 34)),
            ),
            t(TokenKind::STRING, "the", s(p(824, 18, 36), p(826, 18, 38))),
            t(
                TokenKind::STRING,
                "child",
                s(p(828, 18, 40), p(832, 18, 44)),
            ),
            t(TokenKind::CORNER, "└", s(p(848, 19, 15), p(848, 19, 15))),
            t(TokenKind::IT, "it", s(p(858, 19, 19), p(859, 19, 20))),
            t(
                TokenKind::STRING,
                "should",
                s(p(861, 19, 22), p(866, 19, 27)),
            ),
            t(TokenKind::STRING, "emit", s(p(868, 19, 29), p(871, 19, 32))),
            t(TokenKind::STRING, "a", s(p(873, 19, 34), p(873, 19, 34))),
            t(
                TokenKind::STRING,
                "{MultipleChildren}",
                s(p(875, 19, 36), p(892, 19, 53)),
            ),
            t(
                TokenKind::STRING,
                "event",
                s(p(894, 19, 55), p(898, 19, 59)),
            ),
        ];

        assert_eq!(tokens.len(), expected.len());
        assert_eq!(tokens, expected);

        Ok(())
    }
}
