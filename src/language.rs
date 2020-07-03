use crate::{NodeSpec};


// TOKENS, nodes
// token_type, node_type
pub struct Language {
    pub nodes: Vec<NodeSpec>,
    pub lex_error: u16,
    pub language: tree_sitter::Language
}

