use regex::Regex;

/**

UNUSED!!! FOR NOW!!!

**/

#[derive(Clone, Debug)]
pub struct Spec {
    nodes: Vec<TokenSpec>,
    // rules: Vec<Rule>
}



#[derive(Clone, Debug)]
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
        /// you don't want to contains spaces, also don't be empty, they are used as a visual clue for token boundary
        str: &'static str,
        /// separators has some special handling by editor
        is_separator: bool,
        /// don't need user to type space to decide new boundary
        eager: bool,
        /// currently used by highlighter
        semantics: ConstantTokenSemantics
    },
    Regex {
        name: &'static str,
        regex: Regex,
        /// handles disambiguating
        precedence: i32,
        /// can be empty? should be consistent with regex
        can_empty: bool,
        /// can contains space? should be consistent with regex
        can_space: bool,
        /// can contains newline character? should be consistent with regex
        can_newline: bool,
        /// can wrap new line if too long
        can_wrap: bool,
        /// currently used by highlighter
        semantics: FreeTokenSemantics
    }
    // LATER can have shaping settings: logic order or not, complex shaping or not, show codepoint instead...
}

impl TokenSpec {
    pub fn delimiter(str: &'static str) -> TokenSpec {
        TokenSpec::Constant {
            str,
            is_separator: false,
            eager: true,
            semantics: ConstantTokenSemantics::Delimiter
        }
    }

    pub fn separator(str: &'static str) -> TokenSpec {
        TokenSpec::Constant {
            str,
            is_separator: true,
            eager: true,
            semantics: ConstantTokenSemantics::Separator
        }
    }

    pub fn keyword(str: &'static str) -> TokenSpec {
        TokenSpec::Constant {
            str,
            is_separator: false,
            eager: false,
            semantics: ConstantTokenSemantics::Keyword
        }
    }
}

#[derive(Clone, Debug)]
pub enum NodeSpec {
    Token(TokenSpec),
    Tree {
        start: Vec<u16>,
        sep: Vec<u16>,
        end: Vec<u16>
    },
    Compose
}

impl NodeSpec {
    pub fn unwrap_as_token(&self) -> &TokenSpec {
        match self {
            NodeSpec::Token(t) => t,
            NodeSpec::Tree { .. } => panic!(),
            NodeSpec::Compose => panic!(),
        }
    }
}

pub fn unused_node_spec() -> NodeSpec {
    NodeSpec::Token(TokenSpec::Regex {
        name: "",
        regex: Regex::new("").unwrap(),
        can_empty: true,
        precedence: 0,
        can_space: false,
        can_newline: false,
        can_wrap: false,
        semantics: FreeTokenSemantics::Unspecified,
    })
}



//

//

//

//

//

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
