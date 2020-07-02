use tree_sitter::Parser;

pub mod json;


pub trait Language {
    fn new_parser() -> Parser;
}