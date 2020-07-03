use tree_sitter::Parser;
use crate::{Token, NodeType, NodeRole, TokenType};


// TOKENS, nodes
// token_type, node_type
pub trait Language {
    fn new_parser(&self) -> Parser;
    // TODO handle cases like |x|, where left and right is the same
    fn node_role(&self, parent_tp: u16, node_tp: u16) -> NodeRole;
    fn token_type(&self, tp: u16) -> TokenType;
    fn node_type(&self, tp: u16) -> NodeType;
}

