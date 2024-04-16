//! This module implements a translator between a Bulloak tree's High-Level Intermediate
//! Representation (HIR) and a `solang_parser` parse tree (PT). The primary purpose of
//! this module is to facilitate the conversion of a custom HIR into a format that is
//! compatible with the `solang_parser`, which is used for further processing or
//! compilation in the Solidity language context.
//!
//! The translator operates by traversing the HIR in a depth-first order and systematically
//! converting each node into its corresponding PT representation. The translation covers
//! various aspects of the HIR, such as root nodes, contract definitions, function definitions,
//! and comments, each requiring specific handling to maintain the semantics and structure
//! in the resulting PT.
//!
//! The module is structured with a main `Translator` struct that serves as the interface for
//! initiating the translation process. Internally, a `TranslatorI` struct implements the
//! detailed translation logic.

use std::cell::Cell;

use solang_parser::pt::{
    Base, ContractDefinition, ContractPart, ContractTy, Expression, FunctionAttribute,
    FunctionDefinition, FunctionTy, Identifier, IdentifierPath, Import, ImportPath, Loc,
    SourceUnit, SourceUnitPart, Statement, StringLiteral, Type, VariableDeclaration, Visibility,
};

use crate::hir::visitor::Visitor;
use crate::hir::{self, Hir};
use crate::utils::sanitize;

/// The implementation of a translator between a bulloak tree HIR and a
/// `solang_parser` parse tree -- HIR -> PT.
///
/// It visits a HIR in depth-first order an generates a PT
/// as a result.
#[derive(Default)]
pub(crate) struct Translator {
    /// The Solidity version to be used in the pragma directive.
    sol_version: String,
    /// A flag indicating if there is a forge-std dependency
    with_forge_std: bool,
}

impl Translator {
    /// Create a new translator.
    #[must_use]
    pub(crate) fn new(sol_version: &str, with_forge_std: &bool) -> Self {
        Self {
            sol_version: sol_version.to_owned(),
            with_forge_std: with_forge_std.to_owned(),
        }
    }

    /// Translate a HIR to a PT.
    ///
    /// This function is the entry point of the translator.
    #[must_use]
    pub(crate) fn translate(self, hir: &Hir) -> SourceUnit {
        TranslatorI::new(self).translate(hir)
    }
}

/// The internal implementation of the Translator.
struct TranslatorI {
    /// Current byte offset the translator is emitting. Helps in computing locations
    /// for the parse tree nodes.
    offset: Cell<usize>,
    /// The translator state.
    translator: Translator,
}

impl TranslatorI {
    /// Creates a new internal translator.
    fn new(translator: Translator) -> Self {
        Self {
            offset: Cell::new(0),
            translator,
        }
    }

    /// Concrete implementation of the translation from AST to HIR.
    fn translate(mut self, hir: &Hir) -> SourceUnit {
        if let Hir::Root(ref root) = hir {
            self.visit_root(root).unwrap()
        } else {
            SourceUnit(vec![])
        }
    }

    /// Computes a `Loc` out of a string and updates `self.offset` accordingly.
    fn bump(&self, content: &str) -> solang_parser::pt::Loc {
        let start = self.offset.get();
        let end = start + content.len();
        self.offset.set(end);

        Loc::File(0, start, end)
    }

    /// Bumps `self.offset` given a function type and returns the appropriate
    /// `pt::FunctionTy` variant.
    fn translate_function_ty(&self, ty: &hir::FunctionTy) -> FunctionTy {
        match ty {
            hir::FunctionTy::Function => {
                self.bump("function");
                FunctionTy::Function
            }
            hir::FunctionTy::Modifier => {
                self.bump("modifier");
                FunctionTy::Modifier
            }
        }
    }

    /// Bumps `self.offset` given a function identifier and returns the
    /// appropriate `Identifier` instance.
    fn translate_function_id(&self, identifier: &str) -> Identifier {
        let function_name_loc = self.bump(identifier);
        Identifier {
            loc: function_name_loc,
            name: identifier.to_owned(),
        }
    }

    /// Bumps `self.offset` given a modifier and returns the appropriate
    /// `FunctionAttribute::BaseOrModifier`.
    fn translate_modifier(&self, modifier: &str) -> FunctionAttribute {
        let modifier_loc = self.bump(modifier);
        self.bump(" ");
        let modifier = Base {
            loc: modifier_loc,
            name: IdentifierPath {
                loc: modifier_loc,
                identifiers: vec![Identifier {
                    loc: modifier_loc,
                    name: modifier.to_owned(),
                }],
            },
            args: None,
        };

        FunctionAttribute::BaseOrModifier(modifier_loc, modifier)
    }

    /// Generates a list of attributes for a function based on its type in the High-Level
    /// Intermediate Representation (HIR). This function processes the function definition
    /// and constructs a corresponding set of `FunctionAttribute` items, which represent
    /// various attributes of a function in the parse tree (PT), such as visibility and modifiers.
    ///
    /// In the case of a modifier function, an empty vector is returned as modifiers generally
    /// do not have additional attributes in this context. For a regular function, the function
    /// generates the visibility attribute (defaulted to 'external') and includes any modifiers
    /// that are part of the function definition.
    ///
    /// # Arguments
    /// * `function` - A reference to the `FunctionDefinition` node in the HIR. This node contains
    ///   details about the function, including its type and any modifiers associated with it.
    ///
    /// # Returns
    /// A vector of `FunctionAttribute` objects representing the attributes of the function. This
    /// can include visibility attributes and any modifiers that apply to the function.
    fn gen_function_attr(&self, function: &hir::FunctionDefinition) -> Vec<FunctionAttribute> {
        match function.ty {
            hir::FunctionTy::Modifier => vec![],
            hir::FunctionTy::Function => {
                let mut attrs = vec![FunctionAttribute::Visibility(Visibility::External(Some(
                    self.bump("external"),
                )))];
                self.bump(" ");
                if let Some(ref modifiers) = function.modifiers {
                    attrs.extend(modifiers.iter().map(|m| self.translate_modifier(m)));
                };

                attrs
            }
        }
    }

    /// Generates the statements of a modifier function. In the context of this translation, a modifier's
    /// body is represented by a special variable definition. This function creates and returns
    /// a vector of statements that constitute this body.
    ///
    /// The body consists of a single `VariableDefinition` statement with a special identifier `_`,
    /// which is a common convention in Solidity for representing a placeholder in modifier bodies.
    ///
    /// # Returns
    /// A `Vec<Statement>` representing the body of the modifier. This vector contains a single
    /// `VariableDefinition` statement for the `_` placeholder.
    // TODO: After <https://github.com/hyperledger/solang/commit/90b3f7085fe6375e47a29fe01e802f6118c3ad0e>
    // makes it to production, we should update this function and mirror the structure of
    // solang's pt.
    //
    // It is currently a `VariableDefinition` because the formatter does not append
    // a `;` after the variable which causes `forge-fmt` to fail.
    fn gen_modifier_statements(&self) -> Vec<Statement> {
        let mut stmts = Vec::with_capacity(1);
        let variable_loc = self.bump("_");
        let variable = Statement::VariableDefinition(
            variable_loc,
            VariableDeclaration {
                loc: variable_loc,
                ty: Expression::Variable(Identifier {
                    loc: variable_loc,
                    name: "_".to_owned(),
                }),
                storage: None,
                name: None,
            },
            None,
        );
        self.bump(";"); // `;` after `_`.
        stmts.push(variable);

        stmts
    }

    /// Generates the statements of a function by processing its child nodes. This function iterates
    /// through each child node in the provided vector, translating comments into statements
    /// and adding them to the function body.
    ///
    /// A newline character is added after processing all children for proper formatting in the
    /// output. This function is called in the context of translating a function's HIR representation
    /// to its PT (parse tree) counterpart.
    ///
    /// # Arguments
    /// * `children` - A reference to a vector of `Hir` nodes, representing the child nodes
    ///   of a function in the HIR.
    ///
    /// # Returns
    /// A `Result` containing a `Vec<Statement>` representing the translated function body. If
    /// the translation of any child node fails, an error is returned.
    ///
    /// # Errors
    /// Returns an error if any of the child nodes' translation fails.
    fn gen_function_statements(&mut self, children: &Vec<Hir>) -> Result<Vec<Statement>, ()> {
        let mut stmts = Vec::with_capacity(children.len());
        for child in children {
            if let Hir::Statement(statement) = child {
                stmts.push(self.visit_statement(statement)?);
            }
            if let Hir::Comment(comment) = child {
                stmts.push(self.visit_comment(comment)?);
            }
        }

        // If there is at least one child, we add a '\n'
        // for proper formatting.
        if !children.is_empty() {
            self.bump("\n");
        }

        Ok(stmts)
    }

    /// Generates the body of a function or modifier as a sequence of statements. This function
    /// differentiates between function and modifier types and generates the respective bodies
    /// accordingly.
    ///
    /// For a function, the body is generated by processing its child nodes, which may include
    /// comments and other elements specific to the function's logic. For a modifier, a standard
    /// placeholder statement is created, following Solidity's convention for modifiers.
    ///
    /// The function leverages `gen_modifier_statements` for modifiers and
    /// `gen_function_statements` for functions.
    ///
    /// # Arguments
    /// * `function` - A reference to the `FunctionDefinition` node in the High-Level Intermediate
    ///   Representation (HIR). This node contains information about the function, including its
    ///   type (function or modifier) and child nodes.
    ///
    /// # Returns
    /// A `Result` containing a vector of `Statement` objects that represent the body of the
    /// function or modifier. In case of an error during statement generation, an error is returned.
    ///
    /// # Errors
    /// Returns an error if the generation of function statements encounters an issue, such as
    /// failing to process a child node.
    fn gen_function_body(
        &mut self,
        function: &hir::FunctionDefinition,
    ) -> Result<Vec<Statement>, ()> {
        let stmts = match function.ty {
            hir::FunctionTy::Modifier => self.gen_modifier_statements(),
            hir::FunctionTy::Function => {
                if let Some(ref children) = function.children {
                    self.gen_function_statements(children)?
                } else {
                    vec![]
                }
            }
        };

        Ok(stmts)
    }
}

impl Visitor for TranslatorI {
    type RootOutput = SourceUnit;
    type ContractDefinitionOutput = SourceUnitPart;
    type FunctionDefinitionOutput = ContractPart;
    type CommentOutput = Statement;
    type StatementOutput = Statement;
    type Error = ();

    /// Visits the root node of a High-Level Intermediate Representation (HIR) and translates
    /// it into a `SourceUnit` as part of a `solang_parser` parse tree (PT). This function
    /// serves as the entry point for the translation process, converting the top-level structure
    /// of the HIR into a corresponding PT structure.
    ///
    /// The translation involves creating a `SourceUnit`, starting with a pragma directive
    /// based on the translator's Solidity version as well as optional file imports (e.g. forge-std),
    /// if required. It then iterate over each child node within the root.
    /// Each contract definition, is translated and incorporated into the `SourceUnit`.
    ///
    /// # Arguments
    /// * `root` - A reference to the root of the HIR structure, representing the highest level
    ///   of the program's structure.
    ///
    /// # Returns
    /// A `Result` containing the `SourceUnit` if the translation is successful, or an `Error`
    /// otherwise. The `SourceUnit` is a key component of the PT, representing the entire
    /// translated program.
    ///
    /// # Errors
    /// This function may return an error if any part of the translation process fails, such
    /// as issues in converting specific HIR nodes to their PT equivalents.
    fn visit_root(&mut self, root: &hir::Root) -> Result<Self::RootOutput, Self::Error> {
        let mut source_unit = Vec::with_capacity(2);

        let pragma_start = self.offset.get();
        self.bump("pragma");
        self.bump(" ");
        let pragma_ty = Some(Identifier {
            loc: self.bump("solidity"),
            name: "solidity".to_owned(),
        });
        self.bump(" ");
        let pragma_identifier = Some(StringLiteral {
            loc: self.bump(&self.translator.sol_version),
            unicode: false,
            string: self.translator.sol_version.clone(),
        });
        source_unit.push(SourceUnitPart::PragmaDirective(
            Loc::File(0, pragma_start, self.offset.get()),
            pragma_ty,
            pragma_identifier,
        ));
        self.bump(";\n");

        // Add the forge-std's Test import, if needed
        if self.translator.with_forge_std {
            // Getting the relevant offsets for `import {Test} from "forge-std/Test.sol"`
            let loc_import_start = self.offset.get();
            self.bump("import { ");
            let loc_identifier = self.bump("Test");
            self.bump(" } from \"");
            let loc_path = self.bump("forge-std/Test.sol");

            // The import directive `Rename` corresponds to `import {x} from y.sol`
            source_unit.push(SourceUnitPart::ImportDirective(Import::Rename(
                ImportPath::Filename(StringLiteral {
                    loc: loc_path,
                    unicode: false,
                    string: "forge-std/Test.sol".to_string(),
                }),
                vec![(
                    Identifier {
                        loc: loc_identifier,
                        name: "Test".to_string(),
                    },
                    None,
                )],
                Loc::File(0, loc_import_start, loc_path.end()),
            )));

            self.bump("\";\n");
        }

        for child in &root.children {
            if let Hir::ContractDefinition(contract) = child {
                source_unit.push(self.visit_contract(contract)?);
            }
        }

        Ok(SourceUnit(source_unit))
    }

    /// Visits a `ContractDefinition` node within the High-Level Intermediate Representation (HIR)
    /// and translates it into a `SourceUnitPart` for inclusion in the `solang_parser` parse tree (PT).
    /// This function handles the translation of contract definitions, turning them into a format
    /// suitable for the PT, which includes converting each child node of the contract.
    ///
    /// # Arguments
    /// * `contract` - A reference to the `ContractDefinition` node in the HIR, representing a
    ///   single contract within the HIR structure.
    ///
    /// # Returns
    /// A `Result` containing the `SourceUnitPart` representing the translated contract if the
    /// translation is successful, or an `Error` otherwise. The `SourceUnitPart` encapsulates the
    /// contract's PT representation, including its components like functions.
    ///
    /// # Errors
    /// This function may return an error if the translation of any component within the contract
    /// encounters issues, such as failing to translate a function definition.
    fn visit_contract(
        &mut self,
        contract: &hir::ContractDefinition,
    ) -> Result<Self::ContractDefinitionOutput, Self::Error> {
        let contract_start = self.offset.get();
        let contract_ty = ContractTy::Contract(self.bump("contract"));
        self.bump(" ");
        let contract_name = sanitize(&contract.identifier);
        let contract_name = Some(Identifier {
            loc: self.bump(&contract_name),
            name: contract.identifier.clone(),
        });

        let mut contract_base = vec![];

        // If there is an import, inherit the base contract as well
        if self.translator.with_forge_std {
            let base_start = self.offset.get();
            self.bump(" is ");
            let base_loc = self.bump("Test");
            let base_identifier_path = IdentifierPath {
                loc: base_loc,
                identifiers: vec![Identifier {
                    loc: base_loc,
                    name: "Test".to_string(),
                }],
            };

            contract_base = vec![Base {
                loc: Loc::File(0, base_start, base_loc.end()),
                name: base_identifier_path,
                args: None,
            }];
        }
        self.bump(" {"); // `{` after contract identifier and base.

        let mut parts = Vec::with_capacity(contract.children.len());
        for child in &contract.children {
            if let Hir::FunctionDefinition(function) = child {
                parts.push(self.visit_function(function)?);
            }
        }

        let contract_def = ContractDefinition {
            loc: Loc::File(0, contract_start, self.offset.get()),
            name: contract_name,
            ty: contract_ty,
            base: contract_base,
            parts,
        };

        Ok(SourceUnitPart::ContractDefinition(Box::new(contract_def)))
    }

    fn visit_function(
        &mut self,
        function: &hir::FunctionDefinition,
    ) -> Result<Self::FunctionDefinitionOutput, Self::Error> {
        let start_offset = self.offset.get();
        let function_ty = self.translate_function_ty(&function.ty);
        self.bump(" ");
        let function_identifier = self.translate_function_id(&function.identifier);
        let function_id_loc = function_identifier.loc;
        let function_name = Some(function_identifier);
        self.bump("() "); // `(<identifier>) `.
        let attributes = self.gen_function_attr(function);

        let body_start = self.offset.get();
        self.bump("{\n"); // `{` after function attributes.
        let statements = self.gen_function_body(function)?;

        let func_def = FunctionDefinition {
            loc: Loc::File(0, start_offset, body_start - 1),
            ty: function_ty,
            name: function_name,
            name_loc: function_id_loc,
            params: vec![],
            attributes,
            return_not_returns: None,
            returns: vec![],
            body: Some(Statement::Block {
                loc: Loc::File(0, body_start, self.offset.get()),
                unchecked: false,
                statements,
            }),
        };
        self.bump("}\n\n"); // A body ends like this.

        Ok(ContractPart::FunctionDefinition(Box::new(func_def)))
    }

    /// Visits a comment node in the High-Level Intermediate Representation (HIR) and
    /// translates it into a `Statement` for inclusion in the parse tree (PT). This function
    /// handles comments in a unique way by representing them as disguised `VariableDefinition`
    /// statements in the PT.
    ///
    /// The approach involves creating a `VariableDefinition` with a special name and assigning
    /// the comment text to it as a string literal. This method allows for preserving comments
    /// in the PT while maintaining compatibility with the structure of the PT. During a later
    /// phase, we convert the variables back to comments.
    ///
    /// # Arguments
    /// * `comment` - A reference to the `Comment` node in the HIR, containing the comment text.
    ///
    /// # Returns
    /// A `Result` containing the `Statement` representing the comment. The `Statement` is a
    /// `VariableDefinition` with the comment's content. In case of an error, a corresponding
    /// error type is returned.
    ///
    /// # Errors
    /// This function currently does not generate errors but returns a `Result` for consistency
    /// with other visit methods and to allow for error handling in future implementations.
    fn visit_comment(
        &mut self,
        comment: &hir::Comment,
    ) -> Result<Self::CommentOutput, Self::Error> {
        // After exploring several paths forward, the least convoluted way to
        // handle comments is to disguise them as `VariableDefinition` statements.
        //
        // The idea is to remove the extra parts with a search and replace when
        // emitting the parse tree and leave the comment's lexeme as is.
        let definition_start = self.offset.get();
        let declaration_start = definition_start;
        let ty = Expression::Type(self.bump("string"), Type::String);
        self.bump(" "); // ` ` after type.
        let variable_name = "__bulloak_comment__";
        let variable_loc = self.bump(variable_name);
        let declaration_loc = Loc::File(0, declaration_start, self.offset.get());
        let name = Identifier {
            loc: variable_loc,
            name: variable_name.to_owned(),
        };
        let variable = VariableDeclaration {
            loc: declaration_loc,
            ty,
            storage: None,
            name: Some(name),
        };
        self.bump(" = ");
        let comment_loc = self.bump(&format!(r#""{}""#, &comment.lexeme));
        let string_literal = Some(Expression::StringLiteral(vec![StringLiteral {
            loc: comment_loc,
            unicode: false,
            string: comment.lexeme.clone(),
        }]));
        self.bump(";"); // `;` after string literal.
        let definition = Statement::VariableDefinition(
            Loc::File(0, definition_start, self.offset.get()),
            variable,
            string_literal,
        );

        Ok(definition)
    }

    /// Visits a supported statement node and match based on its type.
    fn visit_statement(
        &mut self,
        statement: &hir::Statement,
    ) -> Result<Self::StatementOutput, Self::Error> {
        let start_offset = self.offset.get();

        match statement.ty {
            hir::SupportedStatement::VmSkip => {
                let loc_vm = self.bump("vm");
                self.bump(".");
                let loc_skip = self.bump("skip");
                self.bump("(");
                let loc_arg = self.bump("true");
                self.bump(");");

                let vm_interface = Expression::MemberAccess(
                    Loc::File(0, start_offset, loc_skip.end()),
                    Box::new(Expression::Variable(solang_parser::pt::Identifier {
                        loc: loc_vm,
                        name: "vm".to_owned(),
                    })),
                    solang_parser::pt::Identifier {
                        loc: loc_skip,
                        name: "skip".to_owned(),
                    },
                );

                let vm_skip_arg = vec![Expression::BoolLiteral(loc_arg, true)];

                let vm_skip_call = Expression::FunctionCall(
                    Loc::File(0, loc_skip.start(), loc_arg.end()),
                    Box::new(vm_interface),
                    vm_skip_arg,
                );

                Ok(Statement::Expression(
                    Loc::File(0, start_offset, self.offset.get()),
                    vm_skip_call,
                ))
            }
        }
    }
}
