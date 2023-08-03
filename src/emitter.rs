use std::{collections::HashMap, result};

use crate::{
    ast::{self, Ast},
    utils::capitalize_first_letter,
    visitor::Visitor,
};

/// Solidity code emitter.
pub struct Emitter {
    with_comments: bool,
    indent: usize,
}

impl Emitter {
    pub fn new(with_comments: bool, indent: usize) -> Self {
        Self {
            with_comments,
            indent,
        }
    }

    pub fn emit(self, ast: &ast::Ast, modifiers: &HashMap<String, String>) -> String {
        EmitterI::new(self, modifiers).emit(ast)
    }

    fn indent(&self) -> String {
        " ".repeat(self.indent)
    }
}

struct EmitterI<'a> {
    modifier_stack: Vec<&'a str>,
    modifiers: &'a HashMap<String, String>,
    emitter: Emitter,
}

impl<'a> EmitterI<'a> {
    fn new(emitter: Emitter, modifiers: &'a HashMap<String, String>) -> Self {
        Self {
            modifier_stack: Vec::new(),
            modifiers,
            emitter,
        }
    }

    fn emit(&mut self, ast: &ast::Ast) -> String {
        match ast {
            Ast::Root(ref root) => self.visit_root(root).unwrap(),
            _ => unreachable!(),
        }
    }
}

impl<'a> Visitor for EmitterI<'a> {
    type Output = String;
    type Error = ();

    fn visit_root(&mut self, root: &ast::Root) -> result::Result<Self::Output, Self::Error> {
        let mut emitted = String::new();
        emitted.push_str("pragma solidity [VERSION];\n\n");

        // It's fine to unwrap here because we check that the filename always has an extension.
        let contract_name = root.file_name.split_once('.').unwrap().0;
        let contract_name = capitalize_first_letter(contract_name);
        emitted.push_str(format!("contract {}Test {{\n", contract_name).as_str());

        for condition in &root.asts {
            if let Ast::Condition(condition) = condition {
                emitted.push_str(&self.visit_condition(condition)?);
            }
        }

        emitted.push_str("}\n");

        Ok(emitted)
    }

    fn visit_condition(
        &mut self,
        condition: &ast::Condition,
    ) -> result::Result<Self::Output, Self::Error> {
        let mut emitted = String::new();

        // It's fine to unwrap here because we discover all modifiers in a previous pass.
        let modifier = self.modifiers.get(&condition.title).unwrap();
        self.modifier_stack.push(modifier);

        // We count instead of collecting into a Vec to avoid allocating a Vec for each condition.
        let action_count = condition.asts.iter().filter(|ast| ast.is_action()).count();
        let mut actions = condition.asts.iter().filter(|ast| ast.is_action());

        if action_count > 0 {
            let fn_indentation = self.emitter.indent();
            let fn_body_indentation = fn_indentation.repeat(2);
            // It's fine to unwrap here because we check that no action appears outside of a condition.
            let last_modifier = *self.modifier_stack.last().unwrap();
            let test_name = capitalize_first_letter(last_modifier);

            // If the only action is `it should revert`, we slightly change the function name
            // to reflect this.
            let is_revert = action_count == 1
                && actions.next().is_some_and(|a| {
                    if let Ast::Action(action) = a {
                        action.title == "it should revert"
                    } else {
                        false
                    }
                });
            let function_name = if is_revert {
                format!("testReverts{}", test_name)
            } else {
                format!("test{}", test_name)
            };
            emitted.push_str(format!("{}function {}()\n", fn_indentation, function_name).as_str());
            emitted.push_str(format!("{}external \n", fn_body_indentation).as_str());

            for modifier in &self.modifier_stack {
                emitted.push_str(format!("{}{}\n", fn_body_indentation, modifier).as_str());
            }

            emitted.push_str(format!("{}{{\n", fn_indentation).as_str());
        }

        for action in &condition.asts {
            if let Ast::Action(action) = action {
                emitted.push_str(&self.visit_action(action)?);
            }
        }

        for condition in &condition.asts {
            if let Ast::Condition(condition) = condition {
                emitted.push_str(&self.visit_condition(condition)?);
            }
        }
        emitted.push_str(format!("{}}}\n\n", self.emitter.indent()).as_str());

        self.modifier_stack.pop();

        Ok(emitted)
    }

    fn visit_action(&mut self, action: &ast::Action) -> result::Result<Self::Output, Self::Error> {
        let mut emitted = String::new();

        if self.emitter.with_comments {
            let indentation = self.emitter.indent().repeat(2);
            emitted.push_str(format!("{}// {}\n", indentation, action.title).as_str());
        }

        Ok(emitted)
    }
}
