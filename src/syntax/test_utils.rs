#![allow(missing_docs, unreachable_pub, unused)]
use crate::span::{Position, Span};

#[derive(Clone, Debug)]
pub(crate) struct TestError<K> {
    pub(crate) span: Span,
    pub(crate) kind: K,
}

pub(crate) fn p(offset: usize, line: usize, column: usize) -> Position {
    Position::new(offset, line, column)
}

pub(crate) fn s(start: Position, end: Position) -> Span {
    Span::new(start, end)
}
