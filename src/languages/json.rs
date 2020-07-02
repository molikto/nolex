use tree_sitter::{Parser, Language, Node};
use druid::Color;

use crate::*;

extern "C" { fn tree_sitter_json() -> Language; }

fn language() -> Language {
    unsafe { tree_sitter_json() }
}

pub fn is_tree(node_type: u16) -> bool {
    node_type == 14 || node_type == 16
}

pub fn is_token(tp: u16) -> bool {
    tp >= 1 && tp < 12
}

// TODO handle cases like |x|, where left and right is the same
pub fn token_role(parent_tp: u16, node_tp: u16) -> TokenRole {
    if parent_tp == 14 {
        if node_tp == 1 {
            TokenRole::TreeStart
        } else if node_tp == 2 {
            TokenRole::Sep
        } else if node_tp == 3 {
            TokenRole::TreeEnd
        } else {
            TokenRole::Unspecified
        }
    } else if parent_tp == 16 {
        if node_tp == 5 {
            TokenRole::TreeStart
        } else if node_tp == 2 {
            TokenRole::Sep
        } else if node_tp == 6 {
            TokenRole::TreeEnd
        } else {
            TokenRole::Unspecified
        }
    } else if parent_tp == 15 {
        if node_tp == 2 {
            TokenRole::Sep
        } else {
            TokenRole::Unspecified
        }
    } else {
        TokenRole::Unspecified
    }
}

// parent_type: u8,
pub fn style(token: &Token) -> Color {
    let tp = token.tp;
    if tp == 7 || tp == 8 {
        Color::rgb8(204, 120, 55)
    } else if tp == 9 || tp == 10 || tp == 11 {
        Color::rgb8(106, 135, 89)
    } else {
        Color::rgb8(169, 183, 198)
    }
}

pub fn new_parser() -> Parser {
    let mut parser = Parser::new();
    parser.set_language(language()).unwrap();
    parser
}

