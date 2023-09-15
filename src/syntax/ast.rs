//! The AST for a bulloak tree file.

use crate::span::Span;

/// An Abstract Syntax Tree (AST) that describes the semantic
/// structure of a bulloak tree.
#[derive(Debug, PartialEq, Eq)]
pub enum Ast {
    /// The root node of the AST.
    Root(Root),
    /// A condition node of the AST.
    ///
    /// This node corresponds to a junction in the tree.
    Condition(Condition),
    /// An action node of the AST.
    ///
    /// This node corresponds to a leaf node of the tree.
    Action(Action),
}

impl Ast {
    /// Return the span of this abstract syntax tree.
    #[must_use]
    pub fn span(&self) -> &Span {
        match *self {
            Self::Root(ref x) => &x.span,
            Self::Condition(ref x) => &x.span,
            Self::Action(ref x) => &x.span,
        }
    }

    /// Whether the current node is an `Action` node.
    #[must_use]
    pub fn is_action(&self) -> bool {
        matches!(self, Self::Action(_))
    }
}

/// The root node of the AST.
#[derive(Debug, PartialEq, Eq)]
pub struct Root {
    /// The name that is used for the emitted contract.
    pub contract_name: String,
    /// The span that encompasses this node. It includes
    /// all of its children.
    pub span: Span,
    /// The children AST nodes of this node.
    pub children: Vec<Ast>,
}

/// A condition node of the AST.
#[derive(Debug, PartialEq, Eq)]
pub struct Condition {
    /// The title of this condition.
    ///
    /// For example: "when stuff happens".
    pub title: String,
    /// The span that encompasses this node. It includes
    /// all of its children.
    pub span: Span,
    /// The children AST nodes of this node.
    pub children: Vec<Ast>,
}

/// An action node of the AST.
#[derive(Debug, PartialEq, Eq)]
pub struct Action {
    /// The title of this action.
    ///
    /// For example: "it should revert"
    pub title: String,
    /// The span that encompasses this node.
    pub span: Span,
}
