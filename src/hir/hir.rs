//! Defines a high-level intermediate representation (HIR).

/// A high-level intermediate representation (HIR) that describes
/// the semantic structure of a solidity contract as emitted by `bulloak`.
#[derive(Debug, PartialEq, Eq)]
pub enum Hir {
    /// An abstract root node that does not correspond
    /// to any concrete solidity construct.
    ///
    /// This is used as a sort of "file" boundary since it
    /// is easier to express file-level solidity constraints,
    /// like the pragma directive.
    ///
    /// Note that this means that there can only be a single
    /// root node in any HIR.
    Root(Root),
    /// A contract definition.
    ContractDefinition(ContractDefinition),
    /// A function definition.
    FunctionDefinition(FunctionDefinition),
    /// A comment.
    Comment(Comment),
}

impl Default for Hir {
    fn default() -> Self {
        Hir::Root(Root::default())
    }
}

type Identifier = String;

/// The root HIR node.
///
/// There can only be one root node in any HIR.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Root {
    /// The children HIR nodes of this node.
    pub children: Vec<Hir>,
}

/// A contract definition HIR node.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct ContractDefinition {
    /// The contract name.
    pub identifier: Identifier,
    /// The children HIR nodes of this node.
    pub children: Vec<Hir>,
}

/// A function's type.
///
/// Currently, we only care about regular functions (tests)
/// and modifier functions.
#[derive(Debug, PartialEq, Eq)]
pub enum FunctionTy {
    /// `function`
    Function,
    /// `modifier`
    Modifier,
}

impl Default for FunctionTy {
    fn default() -> Self {
        FunctionTy::Function
    }
}

/// A function definition HIR node.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct FunctionDefinition {
    /// The function name.
    pub identifier: Identifier,
    /// The type of this function.
    pub ty: FunctionTy,
    /// The set of modifiers applied to this function.
    ///
    /// This might be `None` if the function's type
    /// is `FunctionTy::Modifier`.
    pub modifiers: Option<Vec<Identifier>>,
    /// The children HIR nodes of this node.
    pub children: Option<Vec<Hir>>,
}

/// A comment node.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Comment {
    /// The contract name.
    pub lexeme: String,
}
