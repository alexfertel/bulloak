use std::{cmp::Ordering, fmt};

/// Span represents the position information of a single token.
///
/// All span positions are absolute char offsets that can be used on the
/// original tree that was parsed.
#[derive(Clone, Copy, Eq, PartialEq, Default)]
pub struct Span {
    /// The start char offset.
    pub start: Position,
    /// The end char offset.
    pub end: Position,
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Span({:?}, {:?})", self.start, self.end)
    }
}

impl Ord for Span {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.start, &self.end).cmp(&(&other.start, &other.end))
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A single position.
///
/// A position encodes one half of a span, and includes the char offset, line
/// number and column number.
#[derive(Clone, Copy, Eq, PartialEq, Default)]
pub struct Position {
    /// The absolute offset of this position, starting at `0` from the
    /// beginning of the tree.
    ///
    /// Note that this is a `char` offset, which lets us use it when
    /// indexing into the original source string.
    pub offset: usize,
    /// The line number, starting at `1`.
    pub line: usize,
    /// The approximate column number, starting at `1`.
    pub column: usize,
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Position(o: {:?}, l: {:?}, c: {:?})",
            self.offset, self.line, self.column
        )
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Span {
    /// Create a new span with the given positions.
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create a new span using the given position as the start and end.
    pub const fn splat(pos: Position) -> Self {
        Self::new(pos, pos)
    }

    /// Create a new span by replacing the starting position with the one
    /// given.
    pub const fn with_start(self, pos: Position) -> Self {
        Self { start: pos, ..self }
    }

    /// Create a new span by replacing the ending position with the one
    /// given.
    pub const fn with_end(self, pos: Position) -> Self {
        Self { end: pos, ..self }
    }
}

impl Position {
    /// Create a new position with the given information.
    ///
    /// `offset` is the absolute offset of the position, starting at `0` from
    /// the beginning of the tree.
    ///
    /// `line` is the line number, starting at `1`.
    ///
    /// `column` is the approximate column number, starting at `1`.
    pub const fn new(offset: usize, line: usize, column: usize) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }
}
