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
pub enum TerminalRef {
    Constant(&'static str),
    Regex(&'static str)
}

#[derive(Clone, Debug)]
pub enum Syntax {
    // terminal rules
    Terminal(TerminalRef),
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

pub enum TokenRole {
    Unspecified,
    TreeStart,
    TreeEnd,
    Sep
}

