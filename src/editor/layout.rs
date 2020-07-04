use crate::Token;
use druid::piet::{TextLayout, PietTextLayout};

pub struct TokenLayout {
    token: Token,
    margin_left: f64,
    margin_right: f64,
    is_separator: bool,
    layout: PietTextLayout,
}

impl TokenLayout {
    pub fn new(
        token: Token,
        margin_left: f64,
        margin_right: f64,
        is_separator: bool,
        layout: PietTextLayout
    ) -> TokenLayout {
        TokenLayout {
            token, margin_left, margin_right, is_separator, layout
        }
    }
    pub fn layout(&self) -> &PietTextLayout {
        &self.layout
    }
    pub fn width(&self) -> f64 {
        self.layout.width() // TODO trailing whitespace not included
    }
    pub fn tp(&self) -> u16 {
        self.token.tp
    }
    pub fn is_empty(&self) -> bool {
        self.token.str.is_empty()
    }
}

// should always be non-empty
pub struct Line {
    indent: f64,
    tokens: Vec<(f64, TokenLayout)>,
    ascent: f64,
    descent: f64,
    width: f64
}


impl Line {
    pub fn new() -> Line {
        Line { indent: 0.0, tokens: vec![],  width: 0.0, ascent: 0.0, descent: 0.0 }
    }

    pub fn indent(&self) -> f64 { self.indent }
    pub fn ascent(&self) -> f64 { self.ascent }
    pub fn descent(&self) -> f64 { self.descent }
    pub fn tokens(&self) -> &Vec<(f64, TokenLayout)> { &self.tokens }

    pub fn single(token: TokenLayout) -> Line {
        let width = token.width();
        Line { indent: 0.0, tokens: vec![(0.0, token)], width, ascent: 0.0, descent: 0.0 }
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn push(&mut self, t: TokenLayout) {
        if !t.token.str.is_empty() {
            let metrics = t.layout.line_metric(0).unwrap();
            self.ascent = self.ascent.max(metrics.baseline);
            self.descent = self.descent.max(metrics.height - metrics.baseline);
        }
        match self.tokens.last_mut() {
            None => {
            self.width += t.width();
            self.tokens.push((0.0, t));
        },
            Some(last) => {
            let pre_margin = last.1.margin_right;
            let pre_sep = last.1.is_separator;
            let margin= if t.is_separator {
            if pre_sep {
                0.0 // two sep don't have margin
            } else {
                t.margin_left
            }
        } else {
            if pre_sep {
                pre_margin
            } else {
                t.margin_left.max(pre_margin)
            }
        };
            self.width += margin + t.width();
            self.tokens.push((margin, t));
        },
        }
    }

    pub fn append(&mut self, mut other: Line) {
        if !other.is_empty() {
            let ( _, t) = other.tokens.remove(0);
            // TODO the width calculation seems a bit off??
            self.width += other.width - t.width();
            self.push(t);
            self.ascent = self.ascent.max(other.ascent);
            self.descent = self.descent.max(other.descent);
            self.tokens.append(&mut other.tokens);
        }
    }

    pub fn width(&self) -> f64 {
        self.width
    }
}

// should always be multiple line
pub struct Block {
    lines: Vec<Line>
}

impl Block {

    pub fn last_width(&self) -> f64 {
        self.lines.last().unwrap().width
    }

    pub fn width(&mut self) -> f64 {
        let mut w: f64 = 0.0;
        for l in &mut self.lines {
            w = w.max(l.width())
        }
        w
    }

    pub fn new() -> Block {
        Block { lines: vec![Line::new()] }
    }

    pub fn wrap(mut self) -> LayoutResult {
        if self.lines.len() == 1 {
            LayoutResult::Line(self.lines.remove(0))
        } else {
            LayoutResult::Block(self)
        }
    }

    pub fn append_block(&mut self, mut b: Block) {
        if self.lines.is_empty() {
            self.lines = b.lines;
        } else if !b.lines.is_empty() {
            self.lines.last_mut().unwrap().append(b.lines.remove(0));
            self.lines.append(&mut b.lines);
        }
    }

    pub fn append(&mut self, res: LayoutResult) {
        let last = &mut self.lines.last_mut().unwrap();
        match res {
            LayoutResult::Single(a) => {
                last.push(a)
            }
            LayoutResult::Line(l) => last.append(l),
            LayoutResult::Block(b) => {
                self.append_block(b)
            }
        }
    }

    pub fn indent(&mut self, indent: f64) {
        for l in &mut self.lines {
            l.indent += indent;
        }
    }

    pub fn nl(&mut self, indent: f64) {
        let mut line = Line::new();
        line.indent = indent;
        self.lines.push(line);
    }
}

pub enum LayoutResult {
    Single(TokenLayout),
    Line(Line),
    Block(Block),
}

impl LayoutResult {
    pub fn width(&mut self) -> f64 {
        match self {
            LayoutResult::Single(t) => t.width(),
            LayoutResult::Line(l) => l.width(),
            LayoutResult::Block(b) => b.width()
        }
    }

    pub fn to_lines(self) -> Vec<Line> {
        match self {
            LayoutResult::Single(t) => vec![Line::single(t)],
            LayoutResult::Line(l) => vec![l],
            LayoutResult::Block(b) => b.lines
        }
    }

    pub fn to_block(self) -> Block {
        Block { lines: self.to_lines() }
    }
}

