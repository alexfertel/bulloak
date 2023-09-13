use indexmap::IndexMap;
use solang_parser::pt::SourceUnit;

use crate::{check::violation::Violation, hir::visitor::Visitor, hir::Hir, scaffold::modifiers};

/// AST visitor that checks for structural differences between
/// a tree and its corresponding solidity code.
pub struct StructuralMatcher<'ast> {
    //     pub tree_ast: &'ast Ast,
    pub sol_ast: &'ast SourceUnit,
}

// impl<'ast> StructuralMatcher<'ast> {
//     pub(crate) fn new(tree_ast: &'ast Ast, sol_ast: &'ast SourceUnit) -> Self {
//         StructuralMatcher { tree_ast, sol_ast }
//     }
// }

// pub struct StructuralMatcherI<'ast> {
//     matcher: StructuralMatcher<'ast>,
// }

// impl<'ast> StructuralMatcherI<'ast> {
//     fn new(matcher: StructuralMatcher<'ast>) -> Self {
//         StructuralMatcherI { matcher }
//     }

//     // fn check(&mut self) -> Vec<Violation> {
//     //     match self.matcher.tree_ast {
//     //         Ast::(ref root) => self.visit_root(root).unwrap(),
//     //         _ => unreachable!(),
//     //     }
//     // }
// }

// impl Visitor for StructuralMatcherI<'_> {
//     type Output = Vec<Violation>;
//     type Error = ();

//     fn visit_root(&mut self, root: &crate::syntax::ast::Root) -> Result<Self::Output, Self::Error> {
//         let violations = Vec::new();

//         Ok(violations)
//     }

//     fn visit_condition(
//         &mut self,
//         condition: &crate::syntax::ast::Condition,
//     ) -> Result<Self::Output, Self::Error> {
//         todo!()
//     }

//     fn visit_action(
//         &mut self,
//         action: &crate::syntax::ast::Action,
//     ) -> Result<Self::Output, Self::Error> {
//         todo!()
//     }
// }
