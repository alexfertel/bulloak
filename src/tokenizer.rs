use std::{borrow::Borrow, cell::Cell, fmt};

use crate::{
    span::{Position, Span},
    Result,
};

#[derive(PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    span: Span,
    lexeme: String,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token({:?}, {:?})", self.lexeme, self.span)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum TokenKind {
    Vertical,
    Tee,
    Corner,
    Word,
    When,
    It,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TokenStream {
    tokens: Vec<Token>,
}

pub struct Tokenizer {
    pos: Cell<Position>,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            pos: Cell::new(Position::new(0, 1, 1)),
        }
    }

    /// Tokenize the regular expression into an abstract syntax tree.
    pub fn tokenize(&mut self, text: &str) -> Result<TokenStream> {
        TokenizerI::new(self, text).tokenize()
    }

    /// Reset the tokenizer's state.
    fn reset(&self) {
        self.pos.set(Position::new(0, 1, 1));
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
    /// The offset starts at `0` from the beginning of the .tree file.
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
    pub fn tokenize(&self) -> Result<TokenStream> {
        let mut tokens = Vec::new();
        self.tokenizer().reset();

        loop {
            if self.is_eof() {
                break;
            }

            match self.char() {
                ' ' | '\t' | '\r' | '\n' | '─' => {}
                '/' => self.scan_comments(),
                '│' => tokens.push(Token {
                    kind: TokenKind::Vertical,
                    span: self.span(),
                    lexeme: "│".to_string(),
                }),
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
                _ => {
                    let token = self.scan_word()?;
                    tokens.push(token);
                }
            }

            if let None = self.scan() {
                break;
            }
        }

        Ok(TokenStream { tokens })
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
            if self.peek().is_none() || self.peek().is_some_and(|c| c.is_whitespace()) {
                lexeme.push(self.char());
                let kind = match lexeme.as_str() {
                    "when" => TokenKind::When,
                    "it" => TokenKind::It,
                    _ => TokenKind::Word,
                };
                return Ok(Token {
                    kind,
                    span: self.span().with_start(span_start),
                    lexeme,
                });
            } else {
                if is_valid_identifier_char(self.char()) {
                    lexeme.push(self.char());
                    self.scan();
                } else {
                    return Err(format!("Invalid character in identifier: {}", self.char()).into());
                }
            }
        }
    }
}

fn is_valid_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '.' || c == '_'
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Token, TokenKind},
        span::{Position, Span},
        Result,
    };

    use super::Tokenizer;

    #[test]
    fn test_only_filename() -> Result<()> {
        let file_contents = String::from("foo");

        let tokens = Tokenizer::new().tokenize(&file_contents)?.tokens;

        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::Word,
                span: Span::with_length(Position::new(0, 1, 1), 2),
                lexeme: "foo".to_string(),
            }]
        );

        Ok(())
    }

    #[test]
    fn test_only_filename_and_newline() -> Result<()> {
        let file_contents = String::from("foo\n");

        let tokens = Tokenizer::new().tokenize(&file_contents)?.tokens;

        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::Word,
                span: Span::with_length(Position::new(0, 1, 1), 2),
                lexeme: "foo".to_string(),
            }]
        );

        Ok(())
    }

    #[test]
    fn test_one_child() -> Result<()> {
        let file_contents =
            String::from("file.sol\n└── when something bad happens\n   └── it should revert");

        let tokens = Tokenizer::new().tokenize(&file_contents)?.tokens;

        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::Word,
                    span: Span::with_length(Position::new(0, 1, 1), 7),
                    lexeme: "file.sol".to_string(),
                },
                Token {
                    kind: TokenKind::Corner,
                    span: Span::with_length(Position::new(9, 2, 1), 0),
                    lexeme: "└".to_string(),
                },
                Token {
                    kind: TokenKind::When,
                    span: Span::with_length(Position::new(19, 2, 5), 3),
                    lexeme: "when".to_string(),
                },
                Token {
                    kind: TokenKind::Word,
                    span: Span::with_length(Position::new(24, 2, 10), 8),
                    lexeme: "something".to_string(),
                },
                Token {
                    kind: TokenKind::Word,
                    span: Span::with_length(Position::new(34, 2, 20), 2),
                    lexeme: "bad".to_string(),
                },
                Token {
                    kind: TokenKind::Word,
                    span: Span::with_length(Position::new(38, 2, 24), 6),
                    lexeme: "happens".to_string(),
                },
                Token {
                    kind: TokenKind::Corner,
                    span: Span::with_length(Position::new(49, 3, 4), 0),
                    lexeme: "└".to_string(),
                },
                Token {
                    kind: TokenKind::It,
                    span: Span::with_length(Position::new(59, 3, 8), 1),
                    lexeme: "it".to_string(),
                },
                Token {
                    kind: TokenKind::Word,
                    span: Span::with_length(Position::new(62, 3, 11), 5),
                    lexeme: "should".to_string(),
                },
                Token {
                    kind: TokenKind::Word,
                    span: Span::with_length(Position::new(69, 3, 18), 5),
                    lexeme: "revert".to_string(),
                },
            ]
        );

        Ok(())
    }
}
