use druid::im::Vector;

// TODO support large constants by using rope
#[derive(Clone, Debug)]
pub struct Token {
    pub tp: u16,
    pub str: String
}

impl Token {
    pub fn new(tp: u16, str: &'static str) -> Token {
        Token { tp, str: String::from(str) }
    }
}

pub type Tokens = Vector<Token>;
