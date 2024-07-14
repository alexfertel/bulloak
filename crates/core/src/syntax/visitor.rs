//! Defines a trait for visiting a bulloak tree AST in depth-first order.

use crate::syntax::ast;

/// A trait for visiting a tree AST in depth-first order.
///
/// All implementors of `Visitor` must provide a `visit_root` implementation.
/// This is usually the entry point of the visitor, though it is best if this
/// assumption is not held.
pub trait Visitor {
    /// The result of visiting the AST.
    type Output;
    /// An error that might occur when visiting the AST.
    type Error;

    /// This method is called on a root node.
    fn visit_root(
        &mut self,
        root: &ast::Root,
    ) -> Result<Self::Output, Self::Error>;
    /// This method is called on a condition node.
    fn visit_condition(
        &mut self,
        condition: &ast::Condition,
    ) -> Result<Self::Output, Self::Error>;
    /// This method is called on an action node.
    fn visit_action(
        &mut self,
        action: &ast::Action,
    ) -> Result<Self::Output, Self::Error>;
    /// This method is called on an action description node.
    fn visit_description(
        &mut self,
        description: &ast::Description,
    ) -> Result<Self::Output, Self::Error>;
}
