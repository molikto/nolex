
// TODO support large constants by using rope
#[derive(Clone, Debug)]
pub struct Token {
    pub tp: u16,
    pub str: String
}

pub type Tokens = Vec<Token>;

pub fn debug_Tokens_new(tps: &String, strs: Vec<String>) -> Tokens {
    let mut tokens: Vec<Token> = vec![];
    for i in 0.. tps.len() {
        tokens.push(Token { tp : tps.as_bytes()[i].into(), str: strs[i].clone() })
    }
    tokens
}

// TODO support breakable text
pub struct TokenType {
    pub breakable: bool,
}



