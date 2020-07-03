use tree_sitter::Parser;
use crate::{Token, TokenSpec, NodeSpec};


// TOKENS, nodes
// token_type, node_type
pub struct Language {
    pub nodes: Vec<NodeSpec>,
    pub lex_error: u16,
    pub language: tree_sitter::Language
}

