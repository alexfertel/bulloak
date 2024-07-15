//! The AST for a bulloak tree file.

use bulloak_core::span::Span;

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
    /// Additional action description.
    ///
    /// This node can only appear as a child of an action.
    ActionDescription(Description),
}

impl Ast {
    /// Return the span of this abstract syntax tree.
    #[must_use]
    pub fn span(&self) -> &Span {
        match self {
            Self::Root(x) => &x.span,
            Self::Condition(x) => &x.span,
            Self::Action(x) => &x.span,
            Self::ActionDescription(x) => &x.span,
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
    /// For example: "It should revert."
    pub title: String,
    /// The span that encompasses this node.
    pub span: Span,
    /// The children AST nodes of this node.
    ///
    /// For now we only support action description
    /// nodes.
    pub children: Vec<Ast>,
}

/// A description node of the AST.
#[derive(Debug, PartialEq, Eq)]
pub struct Description {
    /// The text of this action.
    ///
    /// For example: "Describe your actions."
    pub text: String,
    /// The span that encompasses this node.
    pub span: Span,
}
