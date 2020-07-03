use regex::Regex;

/**

UNUSED!!! FOR NOW!!!

**/

#[derive(Clone, Debug)]
pub struct Spec {
    tokens: Vec<TokenSpec>,
    rules: Vec<Rule>
}


#[derive(Clone, Debug)]
pub enum TokenSpec {
    Constant(&'static str),
    Regex { name: &'static str, regex: Regex }
}

#[derive(Clone, Debug)]
pub enum TokenRef {
    Constant(&'static str),
    Regex(&'static str)
}

#[derive(Clone, Debug)]
pub enum Syntax {
    // terminal rules
    Token(TokenRef),
    Ref(&'static str),
    Choice(Vec<Syntax>),
    Combine(Vec<Syntax>),
    Sep {
        child: Box<Syntax>,
        sep: Box<Syntax>
    },
    Repeat {
        child: Box<Syntax>,
        min: u32,
        max: u32
    },
    Tree {
        start: Box<Syntax>,
        child: Box<Syntax>,
        sep: Box<Syntax>,
        end: Box<Syntax>
    },
}

#[derive(Clone, Debug)]
pub struct Rule {
    name: &'static str,
    body: Syntax
}

#[derive(Clone, Debug)]
pub enum TokenType {
    Delimiter,
    Keyword,
    Const,
    Unspecified { breakable: bool }
}

#[derive(Clone, Debug)]
pub enum NodeType {
    Unspecified,
    TreeRoot,
    Token(TokenType)
}

#[derive(Clone, Debug)]
pub enum NodeRole {
    Unspecified,
    TreeStart,
    TreeEnd,
    Sep
}

