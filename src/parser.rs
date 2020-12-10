#![warn(clippy::pedantic)]

use std::ops::Range;
use tree_sitter::{Query, Tree};

/// Returns potential tests in the module.
pub fn potential_tests(src: &str) -> Vec<String> {
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_elm::language();
    parser.set_language(language).unwrap();
    let tree = parser.parse(src, None).unwrap();
    get_all_exposed_values_query(&tree, src)
        .into_iter()
        .map(ToString::to_string)
        .collect()
}

lazy_static::lazy_static! {
    static ref EXPOSING_LIST_QUERY: Query = {
        let query_str = "(module_declaration exposing: (exposing_list) @list)";
        Query::new(tree_sitter_elm::language(), query_str).unwrap()
    };
    static ref DOUBLE_DOT_QUERY: Query = {
        let query_str = "((left_parenthesis) . (double_dot))";
        Query::new(tree_sitter_elm::language(), query_str).unwrap()
    };
    static ref EXPOSED_VALUE_QUERY: Query = {
        let query_str = "(exposed_value) @val";
        Query::new(tree_sitter_elm::language(), query_str).unwrap()
    };
    static ref TOP_LEVEL_VALUE_QUERY: Query = {
        let query_str = "(file (value_declaration . (_ . (_) @name)))";
        Query::new(tree_sitter_elm::language(), query_str).unwrap()
    };
}

fn get_all_exposed_values_query<'a>(tree: &'a Tree, source: &'a str) -> Vec<&'a str> {
    match get_exposing_src_range(tree) {
        None => Vec::new(),
        Some(range) => get_explicit_exposed_values_query(tree, source, range)
            .unwrap_or_else(|| get_all_top_level_values_query(tree, source)),
    }
}

fn get_exposing_src_range(tree: &Tree) -> Option<Range<usize>> {
    tree_sitter::QueryCursor::new()
        .matches(&EXPOSING_LIST_QUERY, tree.root_node(), |_| &[])
        .next()
        .map(|m| m.captures[0].node.byte_range())
}

fn get_explicit_exposed_values_query<'a>(
    tree: &'a Tree,
    source: &'a str,
    range: Range<usize>,
) -> Option<Vec<&'a str>> {
    // Restrict the query cursor search to the exposing list
    let mut query_cursor = tree_sitter::QueryCursor::new();
    query_cursor.set_byte_range(range.start, range.end);

    // Check if we have a "exposing (..)"
    if query_cursor
        .matches(&DOUBLE_DOT_QUERY, tree.root_node(), |_| &[])
        .next()
        .is_some()
    {
        return None;
    }

    // Retrieve all exposed values
    Some(
        query_cursor
            .matches(&EXPOSED_VALUE_QUERY, tree.root_node(), |_| &[])
            .map(|m| &source[m.captures[0].node.byte_range()])
            .collect(),
    )
}

fn get_all_top_level_values_query<'a>(tree: &'a Tree, source: &'a str) -> Vec<&'a str> {
    tree_sitter::QueryCursor::new()
        .matches(&TOP_LEVEL_VALUE_QUERY, tree.root_node(), |_| &[])
        .map(|m| &source[m.captures[0].node.byte_range()])
        .collect()
}

#[cfg(test)]
mod tests {

    use tree_sitter::{Parser, Tree};

    fn tree_from_elm(source_code: &str) -> Tree {
        let mut parser = Parser::new();
        let language = tree_sitter_elm::language();
        parser.set_language(language).unwrap();
        parser.parse(source_code, None).unwrap()
    }
    #[test]
    fn smoke() {
        let source_code = "test : Test.Test";
        let tree = tree_from_elm(source_code);
        let root_node = tree.root_node();

        assert_eq!(root_node.kind(), "file");
        assert_eq!(root_node.start_position().column, 0);
        assert_eq!(root_node.end_position().column, 16);
    }
    #[test]
    fn get_explicit_exposed_values() {
        let helper = |source: &str, expected: &Option<Vec<&str>>| {
            let tree = tree_from_elm(source);
            let range = 0..source.len();
            assert_eq!(
                super::get_explicit_exposed_values_query(&tree, source, range).as_ref(),
                expected.as_ref()
            );
        };

        helper("module Main exposing (..)", &None);
        helper("module Main.Pain exposing (..)", &None);
        helper("port module Main.Pain exposing (Int)", &Some(vec![]));
        helper("port module Main.Pain exposing (int)", &Some(vec!["int"]));
        helper(
            "port module Main.Pain exposing (int, Int, test, Test)",
            &Some(vec!["int", "test"]),
        );
        helper(
            "port module Main.Pain exposing (int, Int, {- -}test, Test)",
            &Some(vec!["int", "test"]),
        );
        helper(
            "port module Main.Pain exposing (int, Int, -- comment
    test, Test)",
            &Some(vec!["int", "test"]),
        );
        helper(
            "port module Main.Pain exposing (int, Int,
    test, Test)",
            &Some(vec!["int", "test"]),
        );
        helper(
            "-- some comment
module Main.Pain exposing (int, Int,
    test, Test)",
            &Some(vec!["int", "test"]),
        );
        helper(
            r#"
module{--}Main {-
    {{-}-}-
-}exposing--{-
    ({--}one{--}
    ,
    -- notExport
    two{-{-{-{--}-}{--}-}{-{--}-}-},Type{--}({--}..{--}){--}
    ,    three
    )--
"#,
            &Some(vec!["one", "two", "three"]),
        );
    }
    #[test]
    fn get_all_top_level_values() {
        let helper = |source: &str, expected: &Vec<&str>| {
            let tree = tree_from_elm(source);
            assert_eq!(
                &super::get_all_top_level_values_query(&tree, source),
                expected
            );
        };

        helper("type Test = Thi", &vec![]);
        helper(
            "test = 3
differentTest: Test.Test
differentTest =
    w
",
            &vec!["test", "differentTest"],
        );
        helper(
            "
type Test = Igore

withNestedValues: Test.Test
withNestedValues =
    let
        shouldIgnore = Test.test
    in
    ()
",
            &vec!["withNestedValues"],
        );
        helper(
            "
type Test = Igore

withNestedValues: Test.Test
withNestedValues a =
    let
        shouldIgnore = Test.test
    in
    ()
",
            &vec!["withNestedValues"],
        );
        helper(
            r#"
module Main exposing ( ..)

one="\"{-"
two="""-}
notAThing = something
\"""
notAThing2 = something
"""
three = '"' {- "
notAThing3 = something
-}
four{--}=--{-
    1
five = something
--}
"#,
            &vec!["one", "two", "three", "four", "five"],
        );
    }
}
