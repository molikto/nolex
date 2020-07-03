use tree_sitter::{Parser, Language, Node};
use druid::Color;

use crate::*;


pub struct Json {}

pub const instance: Json = Json { };

extern "C" { fn tree_sitter_json() -> Language; }


fn language() -> Language {
    unsafe { tree_sitter_json() }
}

impl crate::Language for Json {
    fn new_parser(&self) -> Parser {
        let mut parser = Parser::new();
        parser.set_language(language()).unwrap();
        parser
    }

    fn node_role(&self, parent_tp: u16, node_tp: u16) -> NodeRole {
        if parent_tp == 14 {
            if node_tp == 1 {
                NodeRole::TreeStart
            } else if node_tp == 2 {
                NodeRole::Sep
            } else if node_tp == 3 {
                NodeRole::TreeEnd
            } else {
                NodeRole::Unspecified
            }
        } else if parent_tp == 16 {
            if node_tp == 5 {
                NodeRole::TreeStart
            } else if node_tp == 2 {
                NodeRole::Sep
            } else if node_tp == 6 {
                NodeRole::TreeEnd
            } else {
                NodeRole::Unspecified
            }
        } else if parent_tp == 15 {
            if node_tp == 2 {
                NodeRole::Sep
            } else {
                NodeRole::Unspecified
            }
        } else {
            NodeRole::Unspecified
        }
    }

    fn token_type(&self, tp: u16) -> TokenType {
        assert!(tp < 12);
        if tp == 7 || tp == 8 {
            TokenType::Const
            // Color::rgb8(204, 120, 55)
        } else if tp == 9 || tp == 10 || tp == 11 {
            TokenType::Keyword
            // Color::rgb8(106, 135, 89)
        } else {
            TokenType::Delimiter
            // Color::rgb8(169, 183, 198)
        }
    }

    fn node_type(&self, tp: u16) -> NodeType {
        if tp == 14 || tp == 16 {
            NodeType::TreeRoot
        } else if tp >= 1 && tp < 12 {
            NodeType::Token(self.token_type(tp))
        } else {
            NodeType::Unspecified
        }
    }
}
