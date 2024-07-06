//! Visitor helpers to traverse the [solang Solidity Parse
//! Tree](solang_parser::pt).
//!
//! This is based on
//! <https://github.com/foundry-rs/foundry/blob/890bc7a03fd575fbfaf02a8870241f34760e65f1/crates/fmt/src/formatter.rs#L1639>.
#![allow(unused)]

use solang_parser::pt::{
    Base, ContractDefinition, ContractPart, ErrorDefinition, ErrorParameter,
    EventDefinition, EventParameter, Expression, FunctionAttribute,
    FunctionDefinition, Parameter, SourceUnit, SourceUnitPart, Statement,
    StructDefinition, TypeDefinition, VariableAttribute,
};

/// A trait that is invoked while traversing the Solidity Parse Tree.
///
/// This is a subset of the original implementation since we don't
/// need most of it here.
pub(crate) trait Visitor {
    type Output;
    type Error;

    fn visit_source_unit(
        &mut self,
        _source_unit: &mut SourceUnit,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_source_unit_part(
        &mut self,
        part: &mut SourceUnitPart,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_contract(
        &mut self,
        contract: &mut ContractDefinition,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_contract_part(
        &mut self,
        part: &mut ContractPart,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_function(
        &mut self,
        func: &mut FunctionDefinition,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_function_attribute(
        &mut self,
        attribute: &mut FunctionAttribute,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_var_attribute(
        &mut self,
        attribute: &mut VariableAttribute,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_base(
        &mut self,
        base: &mut Base,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_parameter(
        &mut self,
        parameter: &mut Parameter,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_statement(
        &mut self,
        statement: &mut Statement,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_expr(
        &mut self,
        expression: &mut Expression,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_struct(
        &mut self,
        structure: &mut StructDefinition,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_event(
        &mut self,
        event: &mut EventDefinition,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_event_parameter(
        &mut self,
        param: &mut EventParameter,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_error(
        &mut self,
        error: &mut ErrorDefinition,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_error_parameter(
        &mut self,
        param: &mut ErrorParameter,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_type_definition(
        &mut self,
        def: &mut TypeDefinition,
    ) -> Result<Self::Output, Self::Error>;
}
