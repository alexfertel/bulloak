use std::{borrow::Borrow, cell::Cell};

use crate::{
    ast::{Action, Ast, Condition, Root},
    span::{Position, Span},
    tokenizer::{Token, TokenKind},
    Result,
};

pub struct Parser {
    current: Cell<usize>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            current: Cell::new(0),
        }
    }

    pub fn parse(&mut self, tokens: &[Token]) -> Result<Ast> {
        ParserI::new(self, tokens).parse()
    }

    fn reset(&self) {
        self.current.set(0);
    }
}

struct ParserI<'t, P> {
    tokens: &'t [Token],
    parser: P,
}

impl<'t, P: Borrow<Parser>> ParserI<'t, P> {
    fn new(parser: P, tokens: &'t [Token]) -> Self {
        Self { tokens, parser }
    }

    fn parser(&self) -> &Parser {
        self.parser.borrow()
    }

    fn tokens(&self) -> &[Token] {
        self.tokens
    }

    fn current(&self) -> Option<&Token> {
        self.tokens().get(self.parser().current.get())
    }

    fn last(&self) -> Option<&Token> {
        self.tokens().get(self.tokens().len() - 1)
    }

    fn previous(&self) -> Option<&Token> {
        if self.parser().current.get() == 0 {
            return None;
        }
        self.tokens().get(self.parser().current.get() - 1)
    }

    fn peek(&self) -> Option<&Token> {
        if self.parser().current.get() + 1 >= self.tokens().len() {
            return None;
        }
        self.tokens().get(self.parser().current.get() + 1)
    }

    fn consume(&self) -> Option<&Token> {
        if self.parser().current.get() + 1 > self.tokens().len() {
            return None;
        }
        self.parser().current.set(self.parser().current.get() + 1);
        self.tokens().get(self.parser().current.get())
    }

    pub fn parse(&self) -> Result<Ast> {
        self.parser().reset();
        self._parse()
    }

    fn _parse(&self) -> Result<Ast> {
        let current_token = match self.current() {
            Some(current) => current,
            None => return Err("Unexpected end of file".into()),
        };

        match current_token.kind {
            TokenKind::STRING => self.parse_root(),
            TokenKind::TEE => self.parse_corner(current_token),
            TokenKind::CORNER => self.parse_corner(current_token),
            _ => Err(format!("Unexpected token {:?}", current_token).into()),
        }
    }

    fn parse_root(&self) -> Result<Ast> {
        let current_token = self.current().unwrap();
        // A string at the start of the file is the root ast node.
        if current_token.span.start.offset == 0 {
            let mut asts = vec![];

            while let Some(_) = self.consume() {
                // After the root string, we expect a TEE or a CORNER.
                let ast = self._parse()?;
                asts.push(ast);
            }

            let last_span = if asts.len() > 0 {
                asts[asts.len() - 1].span()
            } else {
                &current_token.span
            };

            return Ok(Ast::Root(Root {
                span: Span::new(current_token.span.start, last_span.end),
                asts,
                file_name: current_token.lexeme.clone(),
            }));
        } else {
            // This is a bad state because the only case where we should try to parse a string
            // by itself is at the start of the file. Every other case should be handled as a
            // rhs of a TEE or CORNER.
            return Err(format!(
                "Unexpected STRING token while parsing, found {:?}",
                current_token
            )
            .into());
        }
    }

    fn parse_tee(&self, current_token: &Token) -> Result<Ast> {
        unimplemented!()
    }

    fn parse_corner(&self, current_token: &Token) -> Result<Ast> {
        let next_token = match self.peek() {
            Some(next) => next,
            None => {
                return Err(
                    format!("Unexpected EOF while parsing, found {:?}.", current_token).into(),
                )
            }
        };

        match next_token.kind {
            TokenKind::IT => self.parse_it(current_token),
            TokenKind::WHEN => self.parse_when(current_token),
            _ => Err(format!(
                "Unexpected token {:?} found while expecting a WHEN or an IT",
                next_token
            )
            .into()),
        }
    }

    fn parse_it(&self, start_token: &Token) -> Result<Ast> {
        let title = self.parse_string();
        let previous = self.previous().unwrap();
        Ok(Ast::Action(Action {
            span: Span::new(start_token.span.start, previous.span.end),
            title,
        }))
    }

    fn parse_when(&self, start_token: &Token) -> Result<Ast> {
        let title = self.parse_string();

        let mut asts = vec![];
        while let Some(_) = self.current() {
            // Following a WHEN string, we expect a TEE or a CORNER.
            let ast = self._parse()?;
            asts.push(ast);
            self.consume();
        }

        let previous = self.previous().unwrap();
        Ok(Ast::Condition(Condition {
            span: Span::new(start_token.span.start, previous.span.end),
            title,
            asts,
        }))
    }

    fn parse_string(&self) -> String {
        // Titles always start with one of IT or WHEN.
        let mut string = String::from(self.consume().unwrap().lexeme.clone());

        // Consume all words.
        while let Some(token) = self.consume() {
            match token.kind {
                TokenKind::STRING | TokenKind::IT | TokenKind::WHEN => {
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
    use crate::tokenizer::Tokenizer;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_only_filename() -> Result<()> {
        let tokens = vec![Token {
            kind: TokenKind::STRING,
            lexeme: String::from("foo"),
            span: Span::new(Position::new(0, 1, 1), Position::new(2, 1, 3)),
        }];
        let ast = Parser::new().parse(&tokens)?;

        assert_eq!(
            ast,
            Ast::Root(Root {
                span: Span::new(Position::new(0, 1, 1), Position::new(2, 1, 3)),
                asts: vec![],
                file_name: String::from("foo"),
            })
        );

        Ok(())
    }

    #[test]
    fn test_one_child() -> Result<()> {
        // TODO: Setup tokens by hand instead of relying on the tokenizer.
        let file_contents =
            String::from("file.sol\n└── when something bad happens\n   └── it should revert");

        // Token("file.sol", Span(Position(o: 0, l: 1, c: 1), Position(o: 7, l: 1, c: 8))),
        // Token("└", Span(Position(o: 9, l: 2, c: 1), Position(o: 9, l: 2, c: 1))),
        // Token("when", Span(Position(o: 19, l: 2, c: 5), Position(o: 22, l: 2, c: 8))),
        // Token("something", Span(Position(o: 24, l: 2, c: 10), Position(o: 32, l: 2, c: 18))),
        // Token("bad", Span(Position(o: 34, l: 2, c: 20), Position(o: 36, l: 2, c: 22))),
        // Token("happens", Span(Position(o: 38, l: 2, c: 24), Position(o: 44, l: 2, c: 30))),
        // Token("└", Span(Position(o: 50, l: 3, c: 5), Position(o: 50, l: 3, c: 5))),
        // Token("it", Span(Position(o: 60, l: 3, c: 9), Position(o: 61, l: 3, c: 10))),
        // Token("should", Span(Position(o: 63, l: 3, c: 12), Position(o: 68, l: 3, c: 17))),
        // Token("revert", Span(Position(o: 70, l: 3, c: 19), Position(o: 75, l: 3, c: 24))),
        let tokens = Tokenizer::new().tokenize(&file_contents)?;
        let ast = Parser::new().parse(&tokens)?;

        assert_eq!(
            ast,
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
                file_name: String::from("file.sol"),
            })
        );

        Ok(())
    }
}
