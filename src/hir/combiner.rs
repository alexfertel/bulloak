//! The implementation of a high-level intermediate representation (HIR) combiner.

use crate::syntax::ast::Ast;

use super::{Hir, Root};

/// A high-level intermediate representation (HIR) combiner.
///
/// It takes a vector of HIRs and combines them into a single HIR
/// by appending the function nodes to the root contract node.
pub struct Combiner {}

impl Combiner {
    /// Creates a new combiner.
    #[must_use]
    pub fn new() -> Self {
        Combiner {}
    }

    /// Verifies that the ASTs are valid.
    ///
    /// This function is called before the translation to HIR.
    pub fn verify(&self, asts: &[Ast]) -> Result<(), String> {
        if let Some(Ast::Root(first_root)) = asts.first() {
            let first_contract_name = &first_root.contract_name;
            let first_parts: Vec<&str> = first_contract_name
                .split(|c| c == '.' || c == ':')
                .collect();

            for ast in asts {
                if let Ast::Root(root) = ast {
                    let parts: Vec<&str> =
                        root.contract_name.split(|c| c == '.' || c == ':').collect();
                    if parts[0] != first_parts[0] {
                        return Err(format!(
                            "Contract name mismatch: expected '{}', found '{}'",
                            first_contract_name, root.contract_name
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Combines the translated HIRs into a single HIR.
    ///
    /// This function is called after the ASTs are translated to HIR.
    pub fn combine(&self, hirs: Vec<Hir>) -> Hir {
        // Merge all the HIRs by collecting the function nodes and appending them to the root contract node.
        // @follow-up - how do we deal with duplicate modifiers in separate HIRs?

        let mut root: Root = Root::default();
        for hir in hirs {
            match hir {
                Hir::Root(mut r) => {
                    root.children.append(&mut r.children);
                }
                _ => {}
            }
        }

        Hir::Root(root)
    }
}
