use regex::Regex;
use lazy_static::lazy_static;


use crate::*;

lazy_static! {
  pub static ref INSTANCE: crate::Language = create();
}

fn create() -> crate::Language {
    crate::Language::new(
        vec![
            unused_node_spec(), // 0
            NodeSpec::Token(TokenSpec::delimiter("{")), // 1
            NodeSpec::Token(TokenSpec::separator(",")), // 2
            NodeSpec::Token(TokenSpec::delimiter("}")), // 3
            NodeSpec::Token(TokenSpec::separator(":")), // 4
            NodeSpec::Token(TokenSpec::delimiter("[")), // 5
            NodeSpec::Token(TokenSpec::delimiter("]")), // 6
            NodeSpec::Token(TokenSpec::Regex { // 7
                name: "string",
                regex: Regex::new(".*").unwrap(),
                precedence: 0,
                can_empty: true,
                can_space: true,
                can_newline: true,
                can_wrap: true,
                semantics: FreeTokenSemantics::Literal
            }),
            NodeSpec::Token(TokenSpec::Regex { // 8
                name: "number",
                regex: Regex::new(r#"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?"#).unwrap(), // TODO
                precedence: 10,
                can_empty: false,
                can_space: false,
                can_newline: false,
                can_wrap: false,
                semantics: FreeTokenSemantics::Literal
            }),
            NodeSpec::Token(TokenSpec::keyword("true")), // 9 (11
            NodeSpec::Token(TokenSpec::keyword("false")), // 10 (12
            NodeSpec::Token(TokenSpec::keyword("null")), // 11 (13
            // error node, precedence higher than string, error is handled by a catch all node like literal string
            // unreachable from syntax rules
            // **higher precedence** than string node (precedence only compared within candidate tokens)
            NodeSpec::Token(TokenSpec::Regex { // 12 (14
                name: "",
                regex: Regex::new(".*").unwrap(),
                precedence: 1,
                can_empty: true,
                can_space: true,
                can_newline: true,
                can_wrap: true,
                semantics: FreeTokenSemantics::LexingError
            }),
            NodeSpec::Compose, // 13
            unused_node_spec(), // 14
            NodeSpec::Tree {  // 15 object
                start: vec![1],
                sep: vec![2],
                end: vec![3]
            },
            NodeSpec::Compose,
            NodeSpec::Tree {
                start: vec![5],
                sep: vec![2],
                end: vec![6]
            }
        ],
        language()
    )
}

extern "C" { fn tree_sitter_json() -> tree_sitter::Language; }


fn language() -> tree_sitter::Language {
    unsafe { tree_sitter_json() }
}
