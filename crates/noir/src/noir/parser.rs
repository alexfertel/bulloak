//! Noir code parser using tree-sitter.

use anyhow::{Context, Result};
use tree_sitter::{Node, Parser};

/// Parsed Noir test file.
pub struct ParsedNoirFile {
    /// The source code.
    source: String,
    /// The parsed syntax tree.
    tree: tree_sitter::Tree,
}

/// Information about a test function.
#[derive(Debug, Clone)]
pub struct TestFunction {
    /// The function name.
    pub name: String,
    /// Whether the function has `#[test(should_fail)]` attribute.
    pub has_should_fail: bool,
}

impl ParsedNoirFile {
    /// Parse a Noir file from source code.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn parse(source: &str) -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_noir::language())
            .context("Failed to load Noir grammar")?;

        let tree =
            parser.parse(source, None).context("Failed to parse Noir file")?;

        Ok(Self { source: source.to_string(), tree })
    }

    /// Find all test functions in the file.
    #[must_use]
    pub fn find_test_functions(&self) -> Vec<TestFunction> {
        let mut functions = Vec::new();
        let root_node = self.tree.root_node();

        self.find_test_functions_recursive(root_node, &mut functions);
        functions
    }

    /// Recursively find test functions in a node and its children.
    fn find_test_functions_recursive<'a>(
        &self,
        node: Node<'a>,
        functions: &mut Vec<TestFunction>,
    ) {
        // Check if this node is a function with #[test] attribute
        if node.kind() == "function_definition" {
            if let Some(test_fn) = self.extract_test_function(node) {
                functions.push(test_fn);
            }
        }

        // Recursively check children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.find_test_functions_recursive(child, functions);
        }
    }

    /// Extract test function information from a function node.
    fn extract_test_function<'a>(
        &self,
        node: Node<'a>,
    ) -> Option<TestFunction> {
        // Look for #[test] attribute
        let has_test_attr = self.has_test_attribute(node);
        if !has_test_attr {
            return None;
        }

        // Extract function name
        let name = self.get_function_name(node)?;

        // Check for should_fail
        let has_should_fail = self.has_should_fail_attribute(node);

        Some(TestFunction { name, has_should_fail })
    }

    /// Check if a function has #[test] attribute.
    fn has_test_attribute<'a>(&self, node: Node<'a>) -> bool {
        self.find_attribute(node, "test").is_some()
    }

    /// Check if a function has #[test(should_fail)] attribute.
    fn has_should_fail_attribute<'a>(&self, node: Node<'a>) -> bool {
        if let Some(attr_node) = self.find_attribute(node, "test") {
            // Check if the attribute contains "should_fail"
            let attr_text = self.node_text(attr_node);
            return attr_text.contains("should_fail");
        }
        false
    }

    /// Find a macro/attribute node by name (Noir uses "macro" for attributes).
    fn find_attribute<'a>(
        &self,
        node: Node<'a>,
        attr_name: &str,
    ) -> Option<Node<'a>> {
        // Look for macro nodes before the function
        let mut sibling = node.prev_sibling();
        while let Some(s) = sibling {
            if s.kind() == "macro" {
                let text = self.node_text(s);
                if text.contains(attr_name) {
                    return Some(s);
                }
            } else if s.kind() == "identifier" {
                // Skip "unconstrained" or other modifiers
                let text = self.node_text(s);
                if text == "unconstrained" || text == "pub" {
                    sibling = s.prev_sibling();
                    continue;
                }
                // Stop if we hit an identifier that's not a known modifier
                break;
            } else if s.kind() != "comment" && s.kind() != "line_comment" {
                // Stop if we hit something that's not a macro, comment, or
                // known modifier
                break;
            }
            sibling = s.prev_sibling();
        }

        None
    }

    /// Extract function name from a function node.
    fn get_function_name<'a>(&self, node: Node<'a>) -> Option<String> {
        let mut cursor = node.walk();
        // Find the identifier after "fn" keyword
        let mut found_fn = false;
        for child in node.children(&mut cursor) {
            if child.kind() == "fn" {
                found_fn = true;
            } else if found_fn && child.kind() == "identifier" {
                return Some(self.node_text(child));
            }
        }
        None
    }

    /// Find all helper functions (functions without #[test] attribute).
    #[must_use]
    pub fn find_helper_functions(&self) -> Vec<String> {
        let mut functions = Vec::new();
        let root_node = self.tree.root_node();

        self.find_helper_functions_recursive(root_node, &mut functions);
        functions
    }

    /// Recursively find helper functions in a node and its children.
    fn find_helper_functions_recursive<'a>(
        &self,
        node: Node<'a>,
        functions: &mut Vec<String>,
    ) {
        if node.kind() == "function_definition" {
            // Check if it has #[test] attribute
            if !self.has_test_attribute(node) {
                if let Some(name) = self.get_function_name(node) {
                    functions.push(name);
                }
            }
        }

        // Recursively check children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.find_helper_functions_recursive(child, functions);
        }
    }

    /// Get text content of a node.
    fn node_text<'a>(&self, node: Node<'a>) -> String {
        node.utf8_text(self.source.as_bytes()).unwrap_or("").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_test() {
        let source = r#"
            #[test]
            fn test_something() {
                assert(true);
            }
        "#;

        let parsed = ParsedNoirFile::parse(source).unwrap();
        let test_fns = parsed.find_test_functions();

        assert_eq!(test_fns.len(), 1);
        assert_eq!(test_fns[0].name, "test_something");
        assert!(!test_fns[0].has_should_fail);
    }

    #[test]
    fn test_parse_should_fail() {
        let source = r#"
            #[test(should_fail)]
            fn test_panics() {
                assert(false);
            }
        "#;

        let parsed = ParsedNoirFile::parse(source).unwrap();
        let test_fns = parsed.find_test_functions();

        assert_eq!(test_fns.len(), 1);
        assert_eq!(test_fns[0].name, "test_panics");
        assert!(test_fns[0].has_should_fail);
    }

    #[test]
    fn test_dont_parse_other_macros() {
        let source = r#"
            #[gen_test]
            fn test_panics() {
                assert(false);
            }
            #[gen_test(should_fail)]
            fn test_panics() {
                assert(false);
            }
            #[test_fail(should_fail)]
            fn test_panics() {
                assert(false);
            }
        "#;

        let parsed = ParsedNoirFile::parse(source).unwrap();
        let test_fns = parsed.find_test_functions();

        assert_eq!(test_fns.len(), 0);
    }

    #[test]
    fn test_parse_with_extra_attributes() {
        let source = r#"
            #[test] #[derive(Debug)]
            unconstrained fn test_something() {
                assert(true);
            }

            #[codegen]
            #[test]
            unconstrained fn test_somethingelse() {
                assert(true);
            }

            #[test(should_fail)]
            #[foo]
            unconstrained fn test_panics() {
                assert(false);
            }
        "#;

        let parsed = ParsedNoirFile::parse(source).unwrap();
        let test_fns = parsed.find_test_functions();

        assert_eq!(test_fns.len(), 3);
        assert_eq!(test_fns[0].name, "test_something");
        assert!(!test_fns[0].has_should_fail);

        assert_eq!(test_fns[1].name, "test_somethingelse");
        assert!(!test_fns[1].has_should_fail);

        assert_eq!(test_fns[2].name, "test_panics");
        assert!(test_fns[2].has_should_fail);
    }

    #[test]
    fn test_parse_unconstrained() {
        let source = r#"
            #[test]
            unconstrained fn test_something() {
                assert(true);
            }

            #[test(should_fail)]
            unconstrained fn test_panics() {
                assert(false);
            }
        "#;

        let parsed = ParsedNoirFile::parse(source).unwrap();
        let test_fns = parsed.find_test_functions();

        assert_eq!(test_fns.len(), 2);
        assert_eq!(test_fns[0].name, "test_something");
        assert!(!test_fns[0].has_should_fail);
        assert_eq!(test_fns[1].name, "test_panics");
        assert!(test_fns[1].has_should_fail);
    }

    #[test]
    fn test_find_helper_functions() {
        let source = r#"
            fn helper_function() {
                // helper
            }

            #[test]
            fn test_something() {
                helper_function();
            }
        "#;

        let parsed = ParsedNoirFile::parse(source).unwrap();
        let helpers = parsed.find_helper_functions();

        assert!(!helpers.is_empty());
        assert!(helpers.contains(&"helper_function".to_string()));
    }
}
