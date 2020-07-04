use crate::{NodeSpec, TokenSpec};
use itertools::Itertools;


// TOKENS, nodes
// token_type, node_type
pub struct Language {
    nodes: Vec<NodeSpec>,
    language: tree_sitter::Language,
    lex_error: u16,
    constants: Vec<u16>,
    regexes: Vec<(i32, Vec<u16>)>
}

impl Language {
    pub fn language(&self) -> tree_sitter::Language { self.language }

    pub fn lex_error(&self) -> u16 {
        self.lex_error
    }

    pub fn try_lex(&self, str: &str) -> Option<u16> {
        for &c in &self.constants {
            if self.node(c).as_token().accept(str) {
                return Some(c);
            }
        }
        for (_, rgs) in &self.regexes {
            let accepting: Vec<u16> = rgs.iter().map(|n| *n).filter(|&n| self.node(n).as_token().accept(str)).collect();
            if accepting.len() == 1 {
                return Some(accepting[0]);
            } else if accepting.len() > 0 {
                return None; // TODO ambiguous
            }
        }
        None
    }

    pub fn new(nodes: Vec<NodeSpec>, language: tree_sitter::Language) -> Language {
        let lex_error = nodes.iter().position(|n| match n {
            NodeSpec::Token(t) => t.is_lex_error(),
            _ => false
        }).unwrap() as u16;
        let mut constants: Vec<u16> = vec![];
        let mut regexes: Vec<(i32, u16)> = vec![];
        let mut index: u16 = 0;
        for node in &nodes {
            match node {
                NodeSpec::Token(token) => match token {
                    TokenSpec::Constant { .. } => constants.push(index),
                    TokenSpec::Regex { precedence, .. } => regexes.push((*precedence, index))
                },
                _ => {}
            }
            index += 1;
        }
        regexes.sort_by_key(|n| n.0);
        let mut grouped: Vec<(i32, Vec<u16>)> = vec![];
        for (p, i) in &regexes.into_iter().group_by(|n| n.0) {
            grouped.push((p, i.map(|n| n.1).collect()))
        }
        Language { nodes, language, lex_error, constants, regexes: grouped }
    }
    pub fn node(&self, n: u16) -> &NodeSpec {
        if n == 65535 {
            &NodeSpec::Error
        } else {
            &self.nodes[n as usize]
        }
    }
}

