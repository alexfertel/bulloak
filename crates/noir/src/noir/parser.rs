//! Noir code parser using tree-sitter.

use anyhow::{Context, Result};
use std::sync::LazyLock;
use regex::Regex;
use tree_sitter::{Node, Parser};

use crate::test_structure::{SetupHook, TestFunction};

#[derive(Clone)]
enum Function {
    SetupHook(SetupHook),
    TestFunction(TestFunction),
}

/// Parsed Noir test file.
pub struct ParsedNoirFile {
    /// The source code.
    source: String,
    /// The parsed syntax tree.
    tree: tree_sitter::Tree,
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

    fn find_functions(&self) -> Vec<Function> {
        let mut functions = Vec::new();
        let root_node = self.tree.root_node();

        self.find_functions_recursive(root_node, &mut functions);
        functions
    }

    /// Find all test functions in the file.
    pub fn find_test_functions(&self) -> Vec<TestFunction> {
        self.find_functions()
            .iter()
            .cloned()
            .filter_map(|x| {
                if let Function::TestFunction(f) = x {
                    Some(f)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Find all helper functions (functions without #[test] attribute).
    pub fn find_helper_functions(&self) -> Vec<SetupHook> {
        self.find_functions()
            .iter()
            .cloned()
            .filter_map(|x| {
                if let Function::SetupHook(f) = x {
                    Some(f)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Recursively find test functions in a node and its children.
    fn find_functions_recursive<'a>(
        &self,
        node: Node<'a>,
        functions: &mut Vec<Function>,
    ) {
        // Check if this node is a function with #[test] attribute
        if node.kind() == "function_definition" {
            functions.push(self.extract_function(node));
        }

        // Recursively check children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.find_functions_recursive(child, functions);
        }
    }

    /// Extract test function information from a function node.
    fn extract_function<'a>(&self, node: Node<'a>) -> Function {
        // Look for macro nodes before the function
        let mut sibling = node.prev_sibling();
        while let Some(s) = sibling {
            if s.kind() == "macro" {
                let text = self.node_text(s);
                let (is_test, expect_fail) = parse_test_attribute(&text);
                let name = self
                    .get_function_name(node)
                    .unwrap_or_else(|| panic!("function should have a name"));
                if is_test {
                    return Function::TestFunction(TestFunction {
                        name,
                        expect_fail,
                        setup_hooks: Vec::new(),
                        actions: Vec::new(),
                    });
                } else {
                    sibling = s.prev_sibling();
                    continue;
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
        Function::SetupHook(SetupHook {
            name: self
                .get_function_name(node)
                .unwrap_or_else(|| panic!("function should have a name")),
        })
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

    /// Get text content of a node.
    fn node_text<'a>(&self, node: Node<'a>) -> String {
        node.utf8_text(self.source.as_bytes()).unwrap_or("").to_string()
    }
}

/// it doesn't yet supports stuff like `#[othermacro] #[test]` but that's not used afaik, and the
/// treesitterparser may even produce different attributes for each
/// matches beggining and end of string, so nothing else can be on that line
static TEST_ATTR_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    // ^\s*#\[\s*test\s* -- #[test , with any whitespace
    // (?: non capturing group start
    //   \( literal (
    //   (?: non capturing group start
    //      [^")]|"[^"]*" any character but "), or any string literal.
    //   )* non-capturing group end, zero or more of them.  This allows for should_fail
    //      blocks but doesn't try to parse them.
    //   \) literal )
    // )? matching group end, zero or one of them
    // \s*\]\*$ whitespace, closing bracket, end of string.
    Regex::new(r#"^\s*#\[\s*test\s*(?:\((?:[^")]|"[^"]*")*\))?\s*\]\s*$"#)
        .unwrap()
});

/// does NOT match beggining and end of string, so it should be used after TEST_ATTR_PATTERN
static SHOULD_FAIL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    // branch 1: should_fail (matching group is optional)
    // branch 2: should_fail_with <whitespace> = <whitespace> and quotes with *anything* in between
    Regex::new(r#"should_fail(?:_with\s*=\s*"[^"]*")?"#).unwrap()
});

/// given a macro declaration like  #[test], return if it is a test definition and whether it
/// should expect a revert
fn parse_test_attribute(attribute: &str) -> (bool, bool) {
    if !TEST_ATTR_PATTERN.is_match(attribute) {
        return (false, false);
    }
    let has_should_fail = SHOULD_FAIL_PATTERN.is_match(attribute);
    (true, has_should_fail)
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
        assert!(!test_fns[0].expect_fail);
    }

    #[test]
    fn test_parse_should_fail() {
        let source = r#"
            #[test(should_fail)]
            fn test_panics() {
                assert(false);
            }

            #[test(should_fail_with = "foo")]
            fn test_panics_specifically() {
                assert(false, "foo");
            }
        "#;

        let parsed = ParsedNoirFile::parse(source).unwrap();
        let test_fns = parsed.find_test_functions();

        assert_eq!(test_fns.len(), 2);
        assert_eq!(test_fns[0].name, "test_panics");
        assert!(test_fns[0].expect_fail);

        assert_eq!(test_fns[1].name, "test_panics_specifically");
        assert!(test_fns[1].expect_fail);
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
        assert!(!test_fns[0].expect_fail);

        assert_eq!(test_fns[1].name, "test_somethingelse");
        assert!(!test_fns[1].expect_fail);

        assert_eq!(test_fns[2].name, "test_panics");
        assert!(test_fns[2].expect_fail);
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
        assert!(!test_fns[0].expect_fail);
        assert_eq!(test_fns[1].name, "test_panics");
        assert!(test_fns[1].expect_fail);
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
        assert!(helpers
            .iter()
            .map(|x| x.name.clone())
            .collect::<String>()
            .contains(&"helper_function".to_string()));
    }

    mod parse_test_attribute_tests {
        use super::*;

        #[test]
        fn test_simple_test_attribute() {
            let attr = String::from("#[test]");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(is_test);
            assert!(!should_fail);
        }

        #[test]
        fn test_should_fail_attribute() {
            let attr = String::from("#[test(should_fail)]");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(is_test);
            assert!(should_fail);
        }

        #[test]
        fn test_should_fail_with_message() {
            let attr =
                String::from("#[test(should_fail_with = \"error message\")]");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(is_test);
            assert!(should_fail);
        }

        #[test]
        fn test_should_fail_with_empty_message() {
            let attr = String::from("#[test(should_fail_with = \"\")]");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(is_test);
            assert!(should_fail);
        }

        #[test]
        fn test_non_test_attribute() {
            let attr = String::from("#[derive(Debug)]");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(!is_test);
            assert!(!should_fail);
        }

        #[test]
        fn test_similar_but_not_test_attribute() {
            let attr = String::from("#[gen_test]");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(!is_test);
            assert!(!should_fail);
        }

        #[test]
        fn test_test_prefix_but_not_test() {
            let attr = String::from("#[test_fail]");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(!is_test);
            assert!(!should_fail);
        }

        #[test]
        fn test_test_suffix_but_not_test() {
            let attr = String::from("#[unit_test]");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(!is_test);
            assert!(!should_fail);
        }

        /// probably not even picked up by the parser
        #[test]
        fn test_empty_string() {
            let attr = String::from("");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(!is_test);
            assert!(!should_fail);
        }

        #[test]
        fn test_whitespace_before_attribute() {
            let attr = String::from("#[ test]");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(is_test);
            assert!(!should_fail);
        }

        #[test]
        fn test_whitespace_before_hash() {
            let attr = String::from(" #[test]");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(is_test);
            assert!(!should_fail);
        }

        #[test]
        fn test_whitespace_after_attribute() {
            let attr = String::from("#[test ]");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(is_test);
            assert!(!should_fail);
        }

        #[test]
        fn test_whitespace_after_bracket() {
            let attr = String::from("#[test] ");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(is_test);
            assert!(!should_fail);
        }

        #[test]
        fn test_whitespace_in_attribute() {
            let attr = String::from("#[test( should_fail )]");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(is_test);
            assert!(should_fail);
        }

        /// probably not even picked up by the parser
        #[test]
        fn test_no_brackets() {
            let attr = String::from("test");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(!is_test);
            assert!(!should_fail);
        }

        /// probably not even picked up by the parser
        #[test]
        fn test_missing_hash() {
            let attr = String::from("[test]");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(!is_test);
            assert!(!should_fail);
        }

        /// probably not even picked up by the parser
        #[test]
        fn test_unclosed_bracket() {
            let attr = String::from("#[test");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(!is_test);
            assert!(!should_fail);
        }

        #[test]
        fn test_should_fail_with_special_chars_in_message() {
            let attr = String::from(
                "#[test(should_fail_with = \"error: [1] (foo)\")]",
            );
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(is_test);
            assert!(should_fail);
        }

        #[test]
        fn test_case_sensitivity() {
            let attr = String::from("#[TEST]");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(!is_test);
            assert!(!should_fail);
        }

        #[test]
        fn test_should_fail_case_sensitivity() {
            let attr = String::from("#[test(SHOULD_FAIL)]");
            let (is_test, should_fail) = parse_test_attribute(&attr);
            assert!(is_test);
            assert!(!should_fail);
        }
    }
}
