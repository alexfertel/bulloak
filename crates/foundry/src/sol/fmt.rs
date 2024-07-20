use bulloak_syntax::utils::sanitize;
use once_cell::sync::Lazy;
use regex::Regex;
use solang_parser::pt::{
    Base, ContractDefinition, ContractPart, ErrorDefinition, ErrorParameter,
    EventDefinition, EventParameter, Expression, FunctionAttribute,
    FunctionDefinition, Parameter, SourceUnit, SourceUnitPart, Statement,
    StructDefinition, TypeDefinition, VariableAttribute,
};

use super::visitor::Visitor;

trait Identified {
    fn name(&self) -> String;
}

impl Identified for Base {
    fn name(&self) -> String {
        self.name.identifiers[0].name.clone()
    }
}

pub(crate) struct Formatter;

impl Formatter {
    pub(crate) fn new() -> Self {
        Formatter {}
    }

    pub(crate) fn emit(&mut self, mut pt: SourceUnit) -> String {
        let source = self
            .visit_source_unit(&mut pt)
            .expect("should emit the solidity source");

        cleanup_comments(&source)
    }
}

impl Visitor for Formatter {
    type Error = ();
    type Output = String;

    fn visit_source_unit(
        &mut self,
        source_unit: &mut SourceUnit,
    ) -> Result<Self::Output, Self::Error> {
        let mut parts = vec![];
        for p in &mut source_unit.0 {
            parts.push(self.visit_source_unit_part(p)?);
        }

        Ok(parts.join("\n"))
    }

    fn visit_source_unit_part(
        &mut self,
        part: &mut SourceUnitPart,
    ) -> Result<Self::Output, Self::Error> {
        match part {
            SourceUnitPart::PragmaDirective(_, _, _) => {
                let header = "// SPDX-License-Identifier: UNLICENSED\n";
                let header = format!("{header}{part}");

                Ok(header)
            }
            SourceUnitPart::ContractDefinition(inner) => {
                self.visit_contract(inner)
            }
            SourceUnitPart::FunctionDefinition(inner) => {
                self.visit_function(inner)
            }
            SourceUnitPart::StraySemicolon(_) => Ok(";".to_owned()),
            part => Ok(format!("{part}")),
        }
    }

    fn visit_contract(
        &mut self,
        contract: &mut ContractDefinition,
    ) -> Result<Self::Output, Self::Error> {
        let mut result = String::new();

        result.push_str(&format!("{}", contract.ty));
        result.push(' ');
        if let Some(ref name) = contract.name {
            result.push_str(&sanitize(&format!("{name}")));
            result.push(' ');
        }

        // Include any base contract inherited.
        if !contract.base.is_empty() {
            result.push_str("is ");

            let mut bases = vec![];
            for b in &mut contract.base {
                let base_name = &b.name();
                bases.push(base_name.to_string());
            }
            result.push_str(&bases.join(", "));
            result.push(' ');
        }

        let mut parts = vec![];
        for p in &mut contract.parts {
            parts.push(self.visit_contract_part(p)?);
        }
        result.push('{');
        result.push_str(&parts.join("\n\n"));
        result.push('}');

        Ok(result)
    }

    fn visit_contract_part(
        &mut self,
        part: &mut ContractPart,
    ) -> Result<Self::Output, Self::Error> {
        match part {
            ContractPart::EnumDefinition(inner) => Ok(format!("{inner}")),
            ContractPart::StructDefinition(inner) => Ok(format!("{inner}")),
            ContractPart::EventDefinition(inner) => Ok(format!("{inner}")),
            ContractPart::ErrorDefinition(inner) => Ok(format!("{inner}")),
            ContractPart::FunctionDefinition(inner) => {
                self.visit_function(inner)
            }
            ContractPart::VariableDefinition(inner) => Ok(format!("{inner}")),
            ContractPart::TypeDefinition(inner) => Ok(format!("{inner}")),
            ContractPart::Annotation(inner) => Ok(format!("{inner}")),
            ContractPart::Using(inner) => Ok(format!("{inner}")),
            ContractPart::StraySemicolon(_) => Ok(";".to_owned()),
        }
    }

    fn visit_function(
        &mut self,
        function: &mut FunctionDefinition,
    ) -> Result<Self::Output, Self::Error> {
        let mut result = String::new();

        result.push_str(&format!("{}", function.ty));
        result.push(' ');
        if let Some(ref name) = function.name {
            result.push_str(&format!("{name}"));
        }

        let mut params = vec![];
        for p in &mut function.params {
            if let Some(ref p) = p.1 {
                params.push(format!("{p}"));
            }
        }
        result.push('(');
        result.push_str(&params.join(", "));
        result.push(')');

        if !function.attributes.is_empty() {
            let attributes: Vec<String> = function
                .attributes
                .iter_mut()
                .map(|attr| format!("{attr}"))
                .collect();
            result.push(' ');
            result.push_str(&attributes.join(" "));
        }

        if !function.returns.is_empty() {
            let mut returns = vec![];
            for r in &mut function.returns {
                if let Some(ref r) = r.1 {
                    returns.push(format!("{r}"));
                }
            }
            result.push_str(" returns (");
            result.push_str(&returns.join(" "));
            result.push(')');
        }

        if let Some(ref mut body) = &mut function.body {
            result.push(' ');
            result.push_str(&self.visit_statement(body)?);
        } else {
            result.push(';');
        }

        Ok(result)
    }

    fn visit_statement(
        &mut self,
        statement: &mut Statement,
    ) -> Result<Self::Output, Self::Error> {
        match statement {
            Statement::Block { unchecked, statements, .. } => {
                let mut result = String::new();

                if *unchecked {
                    result.push_str("unchecked ");
                }

                let mut stmts = vec![];
                for s in statements {
                    stmts.push(self.visit_statement(s)?);
                }
                result.push('{');
                result.push('\n');
                result.push_str(&stmts.join("\n"));
                if !stmts.is_empty() {
                    result.push('\n');
                }
                result.push('}');

                Ok(result)
            }
            Statement::Expression(_, ref mut expression) => {
                self.visit_expr(expression)
            }
            statement => Ok(format!("{statement}")),
        }
    }

    fn visit_expr(
        &mut self,
        expression: &mut Expression,
    ) -> Result<Self::Output, Self::Error> {
        match expression {
            Expression::Variable(identifier) => {
                // We need to special case `_`. See
                // <https://github.com/hyperledger/solang/issues/1600>
                if identifier.name == "_" {
                    Ok("_;".to_owned())
                } else {
                    Ok(format!("{identifier}"))
                }
            }
            Expression::FunctionCall(_, _, _) => Ok(format!("{expression};")),
            expression => Ok(format!("{expression}")),
        }
    }

    fn visit_function_attribute(
        &mut self,
        attribute: &mut FunctionAttribute,
    ) -> Result<Self::Output, Self::Error> {
        Ok(format!("{attribute}"))
    }

    fn visit_var_attribute(
        &mut self,
        attribute: &mut VariableAttribute,
    ) -> Result<Self::Output, Self::Error> {
        Ok(format!("{attribute}"))
    }

    fn visit_base(
        &mut self,
        base: &mut Base,
    ) -> Result<Self::Output, Self::Error> {
        Ok(format!("{base}"))
    }

    fn visit_parameter(
        &mut self,
        parameter: &mut Parameter,
    ) -> Result<Self::Output, Self::Error> {
        Ok(format!("{parameter}"))
    }

    fn visit_struct(
        &mut self,
        structure: &mut StructDefinition,
    ) -> Result<Self::Output, Self::Error> {
        Ok(format!("{structure}"))
    }

    fn visit_event(
        &mut self,
        event: &mut EventDefinition,
    ) -> Result<Self::Output, Self::Error> {
        Ok(format!("{event}"))
    }

    fn visit_event_parameter(
        &mut self,
        param: &mut EventParameter,
    ) -> Result<Self::Output, Self::Error> {
        Ok(format!("{param}"))
    }

    fn visit_error(
        &mut self,
        error: &mut ErrorDefinition,
    ) -> Result<Self::Output, Self::Error> {
        Ok(format!("{error}"))
    }

    fn visit_error_parameter(
        &mut self,
        param: &mut ErrorParameter,
    ) -> Result<Self::Output, Self::Error> {
        Ok(format!("{param}"))
    }

    fn visit_type_definition(
        &mut self,
        def: &mut TypeDefinition,
    ) -> Result<Self::Output, Self::Error> {
        Ok(format!("{def}"))
    }
}

/// Converts special `__bulloak_comment__` variables to regular solidity
/// comments.
///
/// Specifically, it looks for patterns matching `string __bulloak_comment__ =
/// "<comment>";` and converts them into `// <comment>` format.
fn cleanup_comments(source: &str) -> String {
    static RE_BULLOAK_COMMENT: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"string __bulloak_comment__ = "(.*)";"#).unwrap()
    });
    RE_BULLOAK_COMMENT.replace_all(source, "// $1").to_string()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::sol::fmt::cleanup_comments;

    #[test]
    fn cleanups_comments() {
        let source = r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;
contract HashPairTest {function test_ShouldNeverRevert() external {
string __bulloak_comment__ = "It should never revert.";
}

modifier whenFirstArgIsSmallerThanSecondArg() {
_;
}

function test_WhenFirstArgIsSmallerThanSecondArg() external whenFirstArgIsSmallerThanSecondArg {
string __bulloak_comment__ = "It should match the result of `keccak256(abi.encodePacked(a,b))`.";
}

function test_WhenFirstArgIsZero() external whenFirstArgIsSmallerThanSecondArg {
string __bulloak_comment__ = "It should do something.";
}

function test_WhenFirstArgIsBiggerThanSecondArg() external {
string __bulloak_comment__ = "It should match the result of `keccak256(abi.encodePacked(b,a))`.";
}}"#;

        let expected = r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;
contract HashPairTest {function test_ShouldNeverRevert() external {
// It should never revert.
}

modifier whenFirstArgIsSmallerThanSecondArg() {
_;
}

function test_WhenFirstArgIsSmallerThanSecondArg() external whenFirstArgIsSmallerThanSecondArg {
// It should match the result of `keccak256(abi.encodePacked(a,b))`.
}

function test_WhenFirstArgIsZero() external whenFirstArgIsSmallerThanSecondArg {
// It should do something.
}

function test_WhenFirstArgIsBiggerThanSecondArg() external {
// It should match the result of `keccak256(abi.encodePacked(b,a))`.
}}"#;
        assert_eq!(expected, cleanup_comments(&source));
    }
}
