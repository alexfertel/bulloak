//! Defines a trait for visiting a high-level intermediate representation (HIR)
//! in depth-first order.

use crate::hir;

/// A trait for visiting a HIR in depth-first order.
pub trait Visitor {
    /// The result of visiting the HIR.
    type Output;
    /// An error that might occur when visiting the HIR.
    type Error;

    /// This method is called on a pragma directive node.
    fn visit_pragma(&mut self, pragma: &hir::PragmaDirective) -> Result<Self::Output, Self::Error>;
    /// This method is called on a contract deifinition node.
    fn visit_contract(
        &mut self,
        contract: &hir::ContractDefinition,
    ) -> Result<Self::Output, Self::Error>;
    /// This method is called on a function definition node.
    fn visit_function(
        &mut self,
        function: &hir::FunctionDefinition,
    ) -> Result<Self::Output, Self::Error>;
}
