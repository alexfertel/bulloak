use crate::ast;

pub trait Visitor {
    type Output;
    type Error;

    fn visit_root(&mut self, root: &ast::Root) -> Result<Self::Output, Self::Error>;
    fn visit_condition(&mut self, condition: &ast::Condition) -> Result<Self::Output, Self::Error>;
    fn visit_action(&mut self, action: &ast::Action) -> Result<Self::Output, Self::Error>;
}
