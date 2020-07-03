use regex::Regex;

/**

UNUSED!!! FOR NOW!!!

**/

#[derive(Clone, Debug)]
pub struct Spec {
    tokens: Vec<TokenSpec>,
    rules: Vec<Rule>
}



#[derive(Clone)]
pub enum ConstantTokenSemantics {
    /// this is special in layout calculation because how they interact with margins,
    /// it always invalidate it's siblings margin because a separator acts as a space itself
    Separator,
    Delimiter,
    Keyword,
}

#[derive(Clone, Debug)]
pub enum FreeTokenSemantics {
    Literal,
    Unspecified
}

#[derive(Clone, Debug)]
pub enum TokenSpec {
    Constant {
        str: &'static str,
        is_separator: bool, // these will not be padded
        semantics: ConstantTokenSemantics
    },
    Regex {
        name: &'static str,
        regex: Regex,
        // info should be consistent with regex
        can_empty: bool,
        can_space: bool,
        breakable: bool,
        semantics: FreeTokenSemantics
    }
    // LATER can have shaping settings: logic order or not, complex shaping or not, show codepoint instead...
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


//
//

#[derive(Clone, Debug)]
pub enum TokenType {
    Separator,
    // these doens't  have any more meanings, we use them mainly to do default highlighting
    Delimiter,
    Keyword,
    Literal, // TODO breakable
    Unspecified
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

