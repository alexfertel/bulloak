use crate::span::Span;

#[derive(Debug, PartialEq, Eq)]
pub enum Ast {
    Root(Root),
    Condition(Condition),
    Action(Action),
}

impl Ast {
    /// Return the span of this abstract syntax tree.
    pub fn span(&self) -> &Span {
        match *self {
            Ast::Root(ref x) => &x.span,
            Ast::Condition(ref x) => &x.span,
            Ast::Action(ref x) => &x.span,
        }
    }

    pub fn is_action(&self) -> bool {
        matches!(self, Ast::Action(_))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Root {
    pub contract_name: String,
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
