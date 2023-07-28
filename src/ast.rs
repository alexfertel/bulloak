use crate::span::Span;

#[derive(Debug, PartialEq, Eq)]
pub enum Ast {
    Empty(Span),
    Root(Root),
    Condition(Condition),
    Action(Action),
}

impl Ast {
    /// Return the span of this abstract syntax tree.
    pub fn span(&self) -> &Span {
        match *self {
            Ast::Empty(ref x) => &x,
            Ast::Root(ref x) => &x.span,
            Ast::Condition(ref x) => &x.span,
            Ast::Action(ref x) => &x.span,
        }
    }

    /// Return true if and only if this Ast is empty.
    pub fn is_empty(&self) -> bool {
        match *self {
            Ast::Empty(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Root {
    pub file_name: String,
    pub span: Span,
    pub asts: Vec<Ast>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Condition {
    pub title: String,
    pub span: Span,
    pub asts: Vec<Ast>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Action {
    pub title: String,
    pub span: Span,
}
