//! Defines a trait for visiting a high-level intermediate representation (HIR)
//! in depth-first order.

use crate::hir;

/// A trait for visiting a HIR in depth-first order.
pub trait Visitor {
    // TODO: Having one associated type per `visit_*` function scales
    // terribly, but for now it's fine. We should use a better abstraction.
    /// The result of visiting a `Root`.
    type RootOutput;
    /// The result of visiting a `ContractDefinition`.
    type ContractDefinitionOutput;
    /// The result of visiting a `FunctionDefinition`.
    type FunctionDefinitionOutput;
    /// The result of visiting a `Comment`.
    type CommentOutput;
    /// The result of visiting an `Expression`.
    type ExpressionOutput;
    /// An error that might occur when visiting the HIR.
    type Error;

    /// Visits the root node of the HIR. This method is typically where the traversal
    /// of the HIR begins.
    ///
    /// # Arguments
    /// * `root` - A reference to the root node of the HIR.
    ///
    /// # Returns
    /// A `Result` containing either the output of visiting the root node or an error.
    fn visit_root(&mut self, root: &hir::Root) -> Result<Self::RootOutput, Self::Error>;
    /// Visits a contract definition node within the HIR.
    ///
    /// # Arguments
    /// * `contract` - A reference to the contract definition node in the HIR.
    ///
    /// # Returns
    /// A `Result` containing either the output of visiting the contract definition node or an error.
    fn visit_contract(
        &mut self,
        contract: &hir::ContractDefinition,
    ) -> Result<Self::ContractDefinitionOutput, Self::Error>;
    /// Visits a function definition node within the HIR.
    ///
    /// # Arguments
    /// * `function` - A reference to the function definition node in the HIR.
    ///
    /// # Returns
    /// A `Result` containing either the output of visiting the function definition node or an error.
    fn visit_function(
        &mut self,
        function: &hir::FunctionDefinition,
    ) -> Result<Self::FunctionDefinitionOutput, Self::Error>;
    /// Visits a comment node within the HIR. This allows for handling comments in the
    /// context of the HIR, potentially for documentation generation or other purposes.
    ///
    /// # Arguments
    /// * `comment` - A reference to the comment node in the HIR.
    ///
    /// # Returns
    /// A `Result` containing either the output of visiting the comment node or an error.
    fn visit_comment(&mut self, comment: &hir::Comment)
        -> Result<Self::CommentOutput, Self::Error>;

    /// Visits an expression node within the HIR. For now, this is a string with the whole expression
    ///     
    /// # Arguments
    /// * `expression` - A reference to the expression node in the HIR.
    ///   
    /// # Returns
    /// A `Result` containing either the output of visiting the expression node or an error.
    fn visit_expression(
        &mut self,
        expression: &hir::Expression,
    ) -> Result<Self::ExpressionOutput, Self::Error>;
}
