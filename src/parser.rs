#![warn(clippy::pedantic)]

use nom::{
    branch::*, bytes::complete::*, character::complete::*, combinator::*, multi::*, sequence::*, *,
};

#[derive(Debug)]
enum Exposing<'a> {
    All,
    Many(Vec<&'a str>),
}

/// Returns potential tests in the module.
/// Warning: does not support effect modules and definition of operators.
pub fn potential_tests(src: &str) -> Vec<&str> {
    alt((parse_file, parse_content))(src)
        .map(|x| x.1)
        .unwrap_or_else(|_| vec![])
}

fn parse_file(input: &str) -> IResult<&str, Vec<&str>> {
    // Parse the module declaration
    let (input, exposing) = preceded(ignore_not_code, module_declaration)(input)?;
    if let Exposing::Many(exposed) = exposing {
        return Ok((input, exposed));
    }

    // Parse the rest of the file
    parse_content(input)
}

fn module_declaration(input: &str) -> IResult<&str, Exposing> {
    // port can be a keyword
    let (input, _) = alt((tag("port"), success("")))(input)?;

    // module keyword potentially surrounded by garbage
    let (input, _) = delimited(ignore_not_code, tag("module"), ignore_not_code)(input)?;

    // identifier of the module
    let (input, _) = take_while(is_allowed_in_module_identifier)(input)?;

    // exposing keyword potentially surrounded by garbage
    let (input, _) = delimited(ignore_not_code, tag("exposing"), ignore_not_code)(input)?;

    // the exposing clause
    alt((
        map(double_dot_expose, |_| Exposing::All),
        map(comma_separated_exposing, Exposing::Many),
    ))(input)
}

// ------------------
// Parsing the exposing

fn comma_separated_exposing(input: &str) -> IResult<&str, Vec<&str>> {
    let exposed_item = delimited(ignore_not_code, take_exposed_identifier, ignore_not_code);
    let (input, items) =
        delimited(tag("("), separated_list1(tag(","), exposed_item), tag(")"))(input)?;
    let potential_tests_items: Vec<&str> =
        items.into_iter().filter(|s| is_potential_test(s)).collect();
    Ok((input, potential_tests_items))
}

fn is_potential_test(identifier: &str) -> bool {
    let first_char = identifier.chars().next().unwrap();
    first_char.is_lowercase()
}

fn take_exposed_identifier(input: &str) -> IResult<&str, &str> {
    let (input, identifier) = take_identifier(input)?;
    let (input, _) = ignore_not_code(input)?;
    let (input, _) = alt((double_dot_expose, success("")))(input)?;
    Ok((input, identifier))
}

fn double_dot_expose(input: &str) -> IResult<&str, &str> {
    delimited(
        preceded(tag("("), ignore_not_code),
        tag(".."),
        terminated(ignore_not_code, tag(")")),
    )(input)
}

// ------------------
// Parsing the content

fn parse_content(input: &str) -> IResult<&str, Vec<&str>> {
    // Parse and ignore all imports
    let (input, _) = ignore_not_code(input)?;
    let ignore_import = terminated(parse_import, ignore_not_code);
    let (input, _) = fold_many0(ignore_import, (), |_, _| ())(input)?;

    // Parse definitions
    let parse_declaration = alt((
        map(parse_type, |_| None),
        map(parse_port, |_| None),
        map(preceded(parse_header, parse_definition), |x| Some(x)),
        map(parse_definition, |x| Some(x)),
    ));
    fold_many0(
        terminated(parse_declaration, ignore_not_code),
        Vec::new(),
        |mut acc: Vec<&str>, item| {
            if let Some(decl) = item {
                acc.push(decl);
            }
            acc
        },
    )(input)
}

fn parse_import(input: &str) -> IResult<&str, ()> {
    let (input, _) = terminated(tag("import"), ignore_not_code)(input)?;
    let (input, _) = take_body(input)?;
    Ok((input, ()))
}

fn parse_type(input: &str) -> IResult<&str, ()> {
    let (input, _) = terminated(tag("type"), ignore_not_code)(input)?;
    let (input, _) = take_body(input)?;
    Ok((input, ()))
}

fn parse_port(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("port")(input)?;
    let (input, _) = space_or_comment(input)?;

    let (input, _) = ignore_not_code(input)?;

    // identifier of the variable
    let (input, identifier) = take_identifier(input)?;

    // take everying until a new line followed by a char that is not a whitespace
    let (input, _) = take_body(input)?;

    Ok((input, identifier))
}

fn parse_header(input: &str) -> IResult<&str, &str> {
    // identifier of the declaration, potentially followed by garbage
    let (input, identifier) = terminated(take_identifier, ignore_not_code)(input)?;

    // : between identifier and type
    let (input, _) = terminated(tag(":"), ignore_not_code)(input)?;

    // type
    let (input, _) = terminated(take_body, ignore_not_code)(input)?;

    Ok((input, identifier))
}

fn parse_definition(input: &str) -> IResult<&str, &str> {
    // identifier of the variable
    let (input, identifier) = terminated(take_identifier, ignore_not_code)(input)?;

    // all the things between identifier and equals sign
    let (input, _) = terminated(take_until("="), tag("="))(input)?;

    // take everying until a new line followed by a char that is not a whitespace
    let (input, _) = take_body(input)?;

    Ok((input, identifier))
}

fn take_identifier(input: &str) -> IResult<&str, &str> {
    take_while1(is_allowed_in_identifier)(input)
}

// Things to ignore

fn ignore_not_code(input: &str) -> IResult<&str, ()> {
    fold_many0(space_or_comment, (), |_, _| ())(input)
}

fn space_or_comment(input: &str) -> IResult<&str, &str> {
    alt((space1, line_ending, block_comment, line_comment))(input)
}

// Handling comments

fn line_comment(input: &str) -> IResult<&str, &str> {
    preceded(tag("--"), not_line_ending)(input)
}

fn block_comment(input: &str) -> IResult<&str, &str> {
    within_recursive("{-", "-}", input)
}

// Warning, the start or end pattern must not contain the other one.
fn within_recursive<'a>(start: &'a str, end: &'a str, input: &'a str) -> IResult<&'a str, &'a str> {
    let (input, _) = tag(start)(input)?;
    let mut rest = input;
    let mut open_count = 1;
    let mut char_count = 0;
    let start_len = start.len();
    let end_len = end.len();
    while open_count > 0 {
        if rest.is_empty() {
            // fail
            return tag("x")("o");
        } else if rest.starts_with(start) {
            rest = &rest[start_len..];
            open_count += 1;
            char_count += start_len;
        } else if rest.starts_with(end) {
            rest = &rest[end_len..];
            open_count -= 1;
            char_count += end_len;
        } else {
            rest = &rest[1..];
            char_count += 1;
        }
    }
    Ok((rest, &input[..char_count - end_len]))
}

// ------------------
// Char filters

fn is_allowed_in_identifier(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_allowed_in_module_identifier(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '.'
}

// ------------------
// Parse strings

fn char_literal(input: &str) -> IResult<&str, &str> {
    delimited(
        char('\''),
        escaped(take_till1(|c| c == '\\' || c == '\''), '\\', anychar),
        char('\''),
    )(input)
}

fn string_literal(input: &str) -> IResult<&str, &str> {
    delimited(
        char('"'),
        alt((
            escaped(take_till1(|c| c == '\\' || c == '"'), '\\', anychar),
            success(""),
        )),
        char('"'),
    )(input)
}

fn multiline_string_literal(input: &str) -> IResult<&str, &str> {
    delimited(
        tag("\"\"\""),
        alt((
            escaped(take_till_escape_or_string_end, '\\', anychar),
            success(""),
        )),
        tag("\"\"\""),
    )(input)
}

fn take_till_escape_or_string_end(input: &str) -> IResult<&str, &str> {
    let mut rest = input;
    let mut count = 0;
    while !rest.is_empty() && !rest.starts_with("\\") && !rest.starts_with("\"\"\"") {
        rest = &rest[1..];
        count += 1;
    }
    if count == 0 {
        tag("x")(rest)
    } else {
        Ok((rest, &input[..count]))
    }
}

// ------------------

fn take_body(input: &str) -> IResult<&str, ()> {
    fold_many0(body_element, (), |_, _| ())(input)
}

fn body_element(input: &str) -> IResult<&str, &str> {
    let forbidden_chars = "'-\"{";
    alt((
        preceded(fold_many0(line_ending, (), |_, _| ()), space1),
        block_comment,
        line_comment,
        char_literal,
        multiline_string_literal,
        string_literal,
        map(one_of(forbidden_chars), |_| ""),
        take_till1(move |c| c == '\n' || forbidden_chars.contains(c)),
    ))(input)
}

#[cfg(test)]
mod tests {
    #[test]
    fn get_explicit_exposed_values() {
        let helper = |source: &str, expected: Option<Vec<&str>>| {
            let exposing = match super::module_declaration(source).unwrap().1 {
                super::Exposing::All => None,
                super::Exposing::Many(v) => Some(v),
            };

            assert_eq!(exposing, expected);
        };

        helper("module Main exposing (..)", None);
        helper("module Main.Pain exposing (..)", None);
        helper("port module Main.Pain exposing (Int)", Some(vec![]));
        helper("port module Main.Pain exposing (int)", Some(vec!["int"]));
        helper(
            "port module Main.Pain exposing (int, Int, test, Test)",
            Some(vec!["int", "test"]),
        );
        helper(
            "port module Main.Pain exposing (int, Int, {- -}test, Test)",
            Some(vec!["int", "test"]),
        );
        helper(
            "port module Main.Pain exposing (int, Int, -- comment
    test, Test)",
            Some(vec!["int", "test"]),
        );
        helper(
            "port module Main.Pain exposing (int, Int,
    test, Test)",
            Some(vec!["int", "test"]),
        );
        helper(
            "-- some comment
module Main.Pain exposing (int, Int,
    test, Test)",
            Some(vec!["int", "test"]),
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
            Some(vec!["one", "two", "three"]),
        );
    }
    #[test]
    fn get_all_top_level_values() {
        let helper = |source: &str, expected: Vec<&str>| {
            let content = super::potential_tests(&source);
            assert_eq!(content, expected);
        };

        helper("type Test = Thi", vec![]);
        helper(
            "test = 3
differentTest: Test.Test
differentTest =
    w
",
            vec!["test", "differentTest"],
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
            vec!["withNestedValues"],
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
            vec!["withNestedValues"],
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
            vec!["one", "two", "three", "four", "five"],
        );
    }
}

#[cfg(test)]
mod nom_tests {
    #[test]
    fn char_literal() {
        let asrt_eq = |input: &str, res| assert_eq!(super::char_literal(input), res);
        assert!(super::char_literal("'").is_err());
        assert!(super::char_literal("''").is_err());
        asrt_eq(r#"'c'a"#, Ok(("a", "c")));
        asrt_eq(r#"'\\'a"#, Ok(("a", "\\\\")));
        asrt_eq(r#"'\''a"#, Ok(("a", "\\'")));
        asrt_eq(r#"'\n'a"#, Ok(("a", "\\n")));
        asrt_eq(r#"'\r'a"#, Ok(("a", "\\r")));
    }
    #[test]
    fn string_literal() {
        let asrt_eq = |input: &str, res| assert_eq!(super::string_literal(input), res);
        assert!(super::string_literal(r#"""#).is_err());
        asrt_eq(r#""toto"a"#, Ok(("a", "toto")));
        asrt_eq(r#""to\"to"a"#, Ok(("a", "to\\\"to")));
        asrt_eq(r#""\"toto"a"#, Ok(("a", "\\\"toto")));
        asrt_eq(r#""\""a"#, Ok(("a", "\\\"")));
        asrt_eq(r#"""a"#, Ok(("a", "")));
        asrt_eq("\"to\nto\"a", Ok(("a", "to\nto")));
        asrt_eq(r#""to\nto"a"#, Ok(("a", "to\\nto")));
        asrt_eq(r#""\""a"#, Ok(("a", "\\\"")));
    }
    #[test]
    fn multiline_string_literal() {
        let asrt_eq = |input: &str, res| assert_eq!(super::multiline_string_literal(input), res);
        assert!(super::multiline_string_literal(r#"""" "#).is_err());
        asrt_eq(r#"""""""a"#, Ok(("a", "")));
        asrt_eq(r#""""toto"""a"#, Ok(("a", "toto")));
        asrt_eq(r#""""to"to"""a"#, Ok(("a", "to\"to")));
        asrt_eq(r#""""to""to"""a"#, Ok(("a", "to\"\"to")));
        asrt_eq(r#""""" """a"#, Ok(("a", "\" ")));
        asrt_eq(r#""""to\"""to"""a"#, Ok(("a", "to\\\"\"\"to")));
        asrt_eq(r#""""to\"""\"to"""a"#, Ok(("a", "to\\\"\"\"\\\"to")));
    }
    #[test]
    fn line_comment() {
        assert!(super::line_comment("a").is_err());
        assert!(super::line_comment("-").is_err());
        assert_eq!(super::line_comment("-- hoho \n"), Ok(("\n", " hoho ")));
        assert_eq!(super::line_comment("-- hoho"), Ok(("", " hoho")));
    }
    #[test]
    fn block_comment() {
        let asrt_eq = |input: &str, res| assert_eq!(super::block_comment(input), res);
        assert!(super::block_comment("").is_err());
        assert!(super::block_comment("a").is_err());
        assert!(super::block_comment("{-").is_err());
        asrt_eq("{- hoho -}", Ok(("", " hoho ")));
        asrt_eq("{- before {- hoho -}-}", Ok(("", " before {- hoho -}")));
        asrt_eq("{-{- hoho -} after -}", Ok(("", "{- hoho -} after ")));
        asrt_eq(
            "{-{- first -} between {- second -}-}",
            Ok(("", "{- first -} between {- second -}")),
        );
    }
}
