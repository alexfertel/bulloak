use std::{borrow::Borrow, cell::Cell, fmt};

use crate::{
    span::{Position, Span},
    Result,
};

#[derive(PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub lexeme: String,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token({:?}, {:?})", self.lexeme, self.span)
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
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            pos: Cell::new(Position::new(0, 1, 1)),
        }
    }

    /// Tokenize the regular expression into an abstract syntax tree.
    pub fn tokenize(&mut self, text: &str) -> Result<Tokens> {
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
    pub fn tokenize(&self) -> Result<Tokens> {
        let mut tokens = Vec::new();
        self.tokenizer().reset();

        loop {
            if self.is_eof() {
                break;
            }

            match self.char() {
                ' ' | '\t' | '\r' | '\n' | '─' | '│' => {}
                '/' => self.scan_comments(),
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
                _ => {
                    let token = self.scan_word()?;
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
            if self.peek().is_none() || self.peek().is_some_and(|c| c.is_whitespace()) {
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
                // FIXME: In a future release, we should detect invalid characters here.
                if is_valid_text_char(self.char()) {
                    lexeme.push(self.char());
                    self.scan();
                } else {
                    return Err(format!("Invalid character in identifier: {}", self.char()).into());
                }
            }
        }
    }
}

fn is_valid_text_char(_c: char) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        span::{Position, Span},
        tokenizer::{Token, TokenKind},
        Result,
    };

    use crate::tokenizer::Tokenizer;

    #[test]
    fn test_only_filename() -> Result<()> {
        let file_contents = String::from("foo");

        let tokens = Tokenizer::new().tokenize(&file_contents)?;

        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::STRING,
                span: Span::with_length(Position::new(0, 1, 1), 2),
                lexeme: "foo".to_string(),
            }]
        );

        Ok(())
    }

    #[test]
    fn test_only_filename_and_newline() -> Result<()> {
        let file_contents = String::from("foo\n");

        let tokens = Tokenizer::new().tokenize(&file_contents)?;

        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::STRING,
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

        let tokens = Tokenizer::new().tokenize(&file_contents)?;

        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::STRING,
                    span: Span::with_length(Position::new(0, 1, 1), 7),
                    lexeme: "file.sol".to_string(),
                },
                Token {
                    kind: TokenKind::CORNER,
                    span: Span::with_length(Position::new(9, 2, 1), 0),
                    lexeme: "└".to_string(),
                },
                Token {
                    kind: TokenKind::WHEN,
                    span: Span::with_length(Position::new(19, 2, 5), 3),
                    lexeme: "when".to_string(),
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::with_length(Position::new(24, 2, 10), 8),
                    lexeme: "something".to_string(),
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::with_length(Position::new(34, 2, 20), 2),
                    lexeme: "bad".to_string(),
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::with_length(Position::new(38, 2, 24), 6),
                    lexeme: "happens".to_string(),
                },
                Token {
                    kind: TokenKind::CORNER,
                    span: Span::with_length(Position::new(49, 3, 4), 0),
                    lexeme: "└".to_string(),
                },
                Token {
                    kind: TokenKind::IT,
                    span: Span::with_length(Position::new(59, 3, 8), 1),
                    lexeme: "it".to_string(),
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::with_length(Position::new(62, 3, 11), 5),
                    lexeme: "should".to_string(),
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::with_length(Position::new(69, 3, 18), 5),
                    lexeme: "revert".to_string(),
                },
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
          ├── when the asset misses the ERC-20 return value
          │  ├── it should create the child
          │  ├── it should perform the ERC-20 transfers
          │  └── it should emit a {MultipleChildren} event
          └── when the asset does not miss the ERC-20 return value
              ├── it should create the child
              └── it should emit a {MultipleChildren} event"#,
        );

        let tokens = Tokenizer::new().tokenize(&file_contents)?;

        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(0, 1, 1), Position::new(22, 1, 23)),
                    lexeme: "multiple_children.t.sol".to_string()
                },
                Token {
                    kind: TokenKind::TEE,
                    span: Span::new(Position::new(24, 2, 1), Position::new(24, 2, 1)),
                    lexeme: "├".to_string()
                },
                Token {
                    kind: TokenKind::WHEN,
                    span: Span::new(Position::new(34, 2, 5), Position::new(37, 2, 8)),
                    lexeme: "when".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(39, 2, 10), Position::new(43, 2, 14)),
                    lexeme: "stuff".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(45, 2, 16), Position::new(50, 2, 21)),
                    lexeme: "called".to_string()
                },
                Token {
                    kind: TokenKind::CORNER,
                    span: Span::new(Position::new(57, 3, 4), Position::new(57, 3, 4)),
                    lexeme: "└".to_string()
                },
                Token {
                    kind: TokenKind::IT,
                    span: Span::new(Position::new(67, 3, 8), Position::new(68, 3, 9)),
                    lexeme: "it".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(70, 3, 11), Position::new(75, 3, 16)),
                    lexeme: "should".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(77, 3, 18), Position::new(82, 3, 23)),
                    lexeme: "revert".to_string()
                },
                Token {
                    kind: TokenKind::CORNER,
                    span: Span::new(Position::new(84, 4, 1), Position::new(84, 4, 1)),
                    lexeme: "└".to_string()
                },
                Token {
                    kind: TokenKind::WHEN,
                    span: Span::new(Position::new(94, 4, 5), Position::new(97, 4, 8)),
                    lexeme: "when".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(99, 4, 10), Position::new(101, 4, 12)),
                    lexeme: "not".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(103, 4, 14), Position::new(107, 4, 18)),
                    lexeme: "stuff".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(109, 4, 20), Position::new(114, 4, 25)),
                    lexeme: "called".to_string()
                },
                Token {
                    kind: TokenKind::TEE,
                    span: Span::new(Position::new(119, 5, 4), Position::new(119, 5, 4)),
                    lexeme: "├".to_string()
                },
                Token {
                    kind: TokenKind::WHEN,
                    span: Span::new(Position::new(129, 5, 8), Position::new(132, 5, 11)),
                    lexeme: "when".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(134, 5, 13), Position::new(136, 5, 15)),
                    lexeme: "the".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(138, 5, 17), Position::new(144, 5, 23)),
                    lexeme: "deposit".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(146, 5, 25), Position::new(151, 5, 30)),
                    lexeme: "amount".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(153, 5, 32), Position::new(154, 5, 33)),
                    lexeme: "is".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(156, 5, 35), Position::new(159, 5, 38)),
                    lexeme: "zero".to_string()
                },
                Token {
                    kind: TokenKind::CORNER,
                    span: Span::new(Position::new(169, 6, 7), Position::new(169, 6, 7)),
                    lexeme: "└".to_string()
                },
                Token {
                    kind: TokenKind::IT,
                    span: Span::new(Position::new(179, 6, 11), Position::new(180, 6, 12)),
                    lexeme: "it".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(182, 6, 14), Position::new(187, 6, 19)),
                    lexeme: "should".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(189, 6, 21), Position::new(194, 6, 26)),
                    lexeme: "revert".to_string()
                },
                Token {
                    kind: TokenKind::CORNER,
                    span: Span::new(Position::new(199, 7, 4), Position::new(199, 7, 4)),
                    lexeme: "└".to_string()
                },
                Token {
                    kind: TokenKind::WHEN,
                    span: Span::new(Position::new(209, 7, 8), Position::new(212, 7, 11)),
                    lexeme: "when".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(214, 7, 13), Position::new(216, 7, 15)),
                    lexeme: "the".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(218, 7, 17), Position::new(224, 7, 23)),
                    lexeme: "deposit".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(226, 7, 25), Position::new(231, 7, 30)),
                    lexeme: "amount".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(233, 7, 32), Position::new(234, 7, 33)),
                    lexeme: "is".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(236, 7, 35), Position::new(238, 7, 37)),
                    lexeme: "not".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(240, 7, 39), Position::new(243, 7, 42)),
                    lexeme: "zero".to_string()
                },
                Token {
                    kind: TokenKind::TEE,
                    span: Span::new(Position::new(251, 8, 7), Position::new(251, 8, 7)),
                    lexeme: "├".to_string()
                },
                Token {
                    kind: TokenKind::WHEN,
                    span: Span::new(Position::new(261, 8, 11), Position::new(264, 8, 14)),
                    lexeme: "when".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(266, 8, 16), Position::new(268, 8, 18)),
                    lexeme: "the".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(270, 8, 20), Position::new(275, 8, 25)),
                    lexeme: "number".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(277, 8, 27), Position::new(281, 8, 31)),
                    lexeme: "count".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(283, 8, 33), Position::new(284, 8, 34)),
                    lexeme: "is".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(286, 8, 36), Position::new(289, 8, 39)),
                    lexeme: "zero".to_string()
                },
                Token {
                    kind: TokenKind::CORNER,
                    span: Span::new(Position::new(302, 9, 10), Position::new(302, 9, 10)),
                    lexeme: "└".to_string()
                },
                Token {
                    kind: TokenKind::IT,
                    span: Span::new(Position::new(312, 9, 14), Position::new(313, 9, 15)),
                    lexeme: "it".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(315, 9, 17), Position::new(320, 9, 22)),
                    lexeme: "should".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(322, 9, 24), Position::new(327, 9, 29)),
                    lexeme: "revert".to_string()
                },
                Token {
                    kind: TokenKind::TEE,
                    span: Span::new(Position::new(335, 10, 7), Position::new(335, 10, 7)),
                    lexeme: "├".to_string()
                },
                Token {
                    kind: TokenKind::WHEN,
                    span: Span::new(Position::new(345, 10, 11), Position::new(348, 10, 14)),
                    lexeme: "when".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(350, 10, 16), Position::new(352, 10, 18)),
                    lexeme: "the".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(354, 10, 20), Position::new(358, 10, 24)),
                    lexeme: "asset".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(360, 10, 26), Position::new(361, 10, 27)),
                    lexeme: "is".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(363, 10, 29), Position::new(365, 10, 31)),
                    lexeme: "not".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(367, 10, 33), Position::new(367, 10, 33)),
                    lexeme: "a".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(369, 10, 35), Position::new(376, 10, 42)),
                    lexeme: "contract".to_string()
                },
                Token {
                    kind: TokenKind::CORNER,
                    span: Span::new(Position::new(389, 11, 10), Position::new(389, 11, 10)),
                    lexeme: "└".to_string()
                },
                Token {
                    kind: TokenKind::IT,
                    span: Span::new(Position::new(399, 11, 14), Position::new(400, 11, 15)),
                    lexeme: "it".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(402, 11, 17), Position::new(407, 11, 22)),
                    lexeme: "should".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(409, 11, 24), Position::new(414, 11, 29)),
                    lexeme: "revert".to_string()
                },
                Token {
                    kind: TokenKind::CORNER,
                    span: Span::new(Position::new(422, 12, 7), Position::new(422, 12, 7)),
                    lexeme: "└".to_string()
                },
                Token {
                    kind: TokenKind::WHEN,
                    span: Span::new(Position::new(432, 12, 11), Position::new(435, 12, 14)),
                    lexeme: "when".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(437, 12, 16), Position::new(439, 12, 18)),
                    lexeme: "the".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(441, 12, 20), Position::new(445, 12, 24)),
                    lexeme: "asset".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(447, 12, 26), Position::new(448, 12, 27)),
                    lexeme: "is".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(450, 12, 29), Position::new(450, 12, 29)),
                    lexeme: "a".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(452, 12, 31), Position::new(459, 12, 38)),
                    lexeme: "contract".to_string()
                },
                Token {
                    kind: TokenKind::TEE,
                    span: Span::new(Position::new(471, 13, 11), Position::new(471, 13, 11)),
                    lexeme: "├".to_string()
                },
                Token {
                    kind: TokenKind::WHEN,
                    span: Span::new(Position::new(481, 13, 15), Position::new(484, 13, 18)),
                    lexeme: "when".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(486, 13, 20), Position::new(488, 13, 22)),
                    lexeme: "the".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(490, 13, 24), Position::new(494, 13, 28)),
                    lexeme: "asset".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(496, 13, 30), Position::new(501, 13, 35)),
                    lexeme: "misses".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(503, 13, 37), Position::new(505, 13, 39)),
                    lexeme: "the".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(507, 13, 41), Position::new(512, 13, 46)),
                    lexeme: "ERC-20".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(514, 13, 48), Position::new(519, 13, 53)),
                    lexeme: "return".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(521, 13, 55), Position::new(525, 13, 59)),
                    lexeme: "value".to_string()
                },
                Token {
                    kind: TokenKind::TEE,
                    span: Span::new(Position::new(542, 14, 14), Position::new(542, 14, 14)),
                    lexeme: "├".to_string()
                },
                Token {
                    kind: TokenKind::IT,
                    span: Span::new(Position::new(552, 14, 18), Position::new(553, 14, 19)),
                    lexeme: "it".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(555, 14, 21), Position::new(560, 14, 26)),
                    lexeme: "should".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(562, 14, 28), Position::new(567, 14, 33)),
                    lexeme: "create".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(569, 14, 35), Position::new(571, 14, 37)),
                    lexeme: "the".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(573, 14, 39), Position::new(577, 14, 43)),
                    lexeme: "child".to_string()
                },
                Token {
                    kind: TokenKind::TEE,
                    span: Span::new(Position::new(594, 15, 14), Position::new(594, 15, 14)),
                    lexeme: "├".to_string()
                },
                Token {
                    kind: TokenKind::IT,
                    span: Span::new(Position::new(604, 15, 18), Position::new(605, 15, 19)),
                    lexeme: "it".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(607, 15, 21), Position::new(612, 15, 26)),
                    lexeme: "should".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(614, 15, 28), Position::new(620, 15, 34)),
                    lexeme: "perform".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(622, 15, 36), Position::new(624, 15, 38)),
                    lexeme: "the".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(626, 15, 40), Position::new(631, 15, 45)),
                    lexeme: "ERC-20".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(633, 15, 47), Position::new(641, 15, 55)),
                    lexeme: "transfers".to_string()
                },
                Token {
                    kind: TokenKind::CORNER,
                    span: Span::new(Position::new(658, 16, 14), Position::new(658, 16, 14)),
                    lexeme: "└".to_string()
                },
                Token {
                    kind: TokenKind::IT,
                    span: Span::new(Position::new(668, 16, 18), Position::new(669, 16, 19)),
                    lexeme: "it".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(671, 16, 21), Position::new(676, 16, 26)),
                    lexeme: "should".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(678, 16, 28), Position::new(681, 16, 31)),
                    lexeme: "emit".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(683, 16, 33), Position::new(683, 16, 33)),
                    lexeme: "a".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(685, 16, 35), Position::new(702, 16, 52)),
                    lexeme: "{MultipleChildren}".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(704, 16, 54), Position::new(708, 16, 58)),
                    lexeme: "event".to_string()
                },
                Token {
                    kind: TokenKind::CORNER,
                    span: Span::new(Position::new(720, 17, 11), Position::new(720, 17, 11)),
                    lexeme: "└".to_string()
                },
                Token {
                    kind: TokenKind::WHEN,
                    span: Span::new(Position::new(730, 17, 15), Position::new(733, 17, 18)),
                    lexeme: "when".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(735, 17, 20), Position::new(737, 17, 22)),
                    lexeme: "the".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(739, 17, 24), Position::new(743, 17, 28)),
                    lexeme: "asset".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(745, 17, 30), Position::new(748, 17, 33)),
                    lexeme: "does".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(750, 17, 35), Position::new(752, 17, 37)),
                    lexeme: "not".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(754, 17, 39), Position::new(757, 17, 42)),
                    lexeme: "miss".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(759, 17, 44), Position::new(761, 17, 46)),
                    lexeme: "the".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(763, 17, 48), Position::new(768, 17, 53)),
                    lexeme: "ERC-20".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(770, 17, 55), Position::new(775, 17, 60)),
                    lexeme: "return".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(777, 17, 62), Position::new(781, 17, 66)),
                    lexeme: "value".to_string()
                },
                Token {
                    kind: TokenKind::TEE,
                    span: Span::new(Position::new(797, 18, 15), Position::new(797, 18, 15)),
                    lexeme: "├".to_string()
                },
                Token {
                    kind: TokenKind::IT,
                    span: Span::new(Position::new(807, 18, 19), Position::new(808, 18, 20)),
                    lexeme: "it".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(810, 18, 22), Position::new(815, 18, 27)),
                    lexeme: "should".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(817, 18, 29), Position::new(822, 18, 34)),
                    lexeme: "create".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(824, 18, 36), Position::new(826, 18, 38)),
                    lexeme: "the".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(828, 18, 40), Position::new(832, 18, 44)),
                    lexeme: "child".to_string()
                },
                Token {
                    kind: TokenKind::CORNER,
                    span: Span::new(Position::new(848, 19, 15), Position::new(848, 19, 15)),
                    lexeme: "└".to_string()
                },
                Token {
                    kind: TokenKind::IT,
                    span: Span::new(Position::new(858, 19, 19), Position::new(859, 19, 20)),
                    lexeme: "it".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(861, 19, 22), Position::new(866, 19, 27)),
                    lexeme: "should".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(868, 19, 29), Position::new(871, 19, 32)),
                    lexeme: "emit".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(873, 19, 34), Position::new(873, 19, 34)),
                    lexeme: "a".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(875, 19, 36), Position::new(892, 19, 53)),
                    lexeme: "{MultipleChildren}".to_string()
                },
                Token {
                    kind: TokenKind::STRING,
                    span: Span::new(Position::new(894, 19, 55), Position::new(898, 19, 59)),
                    lexeme: "event".to_string()
                },
            ]
        );

        Ok(())
    }
}
