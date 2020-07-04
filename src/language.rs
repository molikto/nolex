use crate::{NodeSpec};


// TOKENS, nodes
// token_type, node_type
pub struct Language {
    nodes: Vec<NodeSpec>,
    language: tree_sitter::Language
}

impl Language {
    pub fn language(&self) -> tree_sitter::Language { self.language }

    pub fn new(nodes: Vec<NodeSpec>, language: tree_sitter::Language) -> Language {
        Language { nodes, language }
    }
    pub fn node(&self, n: u16) -> &NodeSpec {
        if n == 65535 {
            &NodeSpec::Error
        } else {
            &self.nodes[n as usize]
        }
    }
}

