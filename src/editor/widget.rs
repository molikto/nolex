use druid::piet::{FontBuilder, Text, TextLayoutBuilder, TextLayout, PietFont, PietTextLayout, PietText};
use druid::widget::prelude::*;
use druid::{Point, Color};
use tree_sitter::{Parser, Node, Tree};

use crate::*;

// TODO partial layout by using layout focus & offset etc. handle scroll ourselves
// TODO reuse text layout for commonly created strs with same attribute?

struct TokenLayout {
    token: Token,
    margin_left: f64,
    margin_right: f64,
    is_separator: bool,
    layout: PietTextLayout,
}

impl TokenLayout {
    fn width(&self) -> f64 {
        self.layout.width()
    }
}

// should always be non-empty
struct Line {
    indent: f64,
    tokens: Vec<(f64, TokenLayout)>,
    width: f64
}


impl Line {
    fn new() -> Line {
        Line { indent: 0.0, tokens: vec![],  width: 0.0 }
    }

    fn single(token: TokenLayout) -> Line {
        let width = token.width();
        Line { indent: 0.0, tokens: vec![(0.0, token)], width }
    }

    fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    fn push(&mut self, t: TokenLayout) {
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

    fn append(&mut self, mut other: Line) {
        if !other.is_empty() {
            let ( _, t) = other.tokens.remove(0);
            self.push(t);
            self.tokens.append(&mut other.tokens);
        }
    }

    fn width(&mut self) -> f64 {
        self.width
    }
}

// should always be multiple line
struct Block {
    lines: Vec<Line>
}

impl Block {
    fn width(&mut self) -> f64 {
        let mut w: f64 = 0.0;
        for l in &mut self.lines {
            w = w.max(l.width())
        }
        w
    }

    fn new() -> Block {
        Block { lines: vec![Line::new()] }
    }

    fn wrap(mut self) -> LayoutResult {
        if self.lines.len() == 1 {
            LayoutResult::Line(self.lines.remove(0))
        } else {
            LayoutResult::Block(self)
        }
    }

    fn append_block(&mut self, mut b: Block) {
        let last = &mut self.lines.last_mut().unwrap();
        if last.is_empty() {
            self.lines.remove(self.lines.len() - 1);
        }
        self.lines.append(&mut b.lines);
    }

    fn append(&mut self, res: LayoutResult) {
        let mut last = &mut self.lines.last_mut().unwrap();
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

    fn indent(&mut self, indent: f64) {
        for l in &mut self.lines {
            l.indent += indent;
        }
    }

    fn nl(&mut self) {
        self.lines.push(Line::new())
    }
}

enum LayoutResult {
    Single(TokenLayout),
    Line(Line),
    Block(Block),
}

impl LayoutResult {
    fn width(&mut self) -> f64 {
        match self {
            LayoutResult::Single(t) => t.width(),
            LayoutResult::Line(l) => l.width(),
            LayoutResult::Block(b) => b.width()
        }
    }

    fn to_lines(self) -> Vec<Line> {
        match self {
            LayoutResult::Single(t) => vec![Line::single(t)],
            LayoutResult::Line(l) => vec![l],
            LayoutResult::Block(b) => b.lines
        }
    }

    fn to_block(self) -> Block {
        Block { lines: self.to_lines() }
    }
}


pub struct EditorState {
    version: u64,
    language: &'static Language,
    parser: Parser,
    tokens: Tokens,
    tree: Tree,
    font: Option<PietFont>,
    max_width: f64,
    layout: Vec<Line>,
}

impl EditorState {
    pub fn new() -> EditorState {
        let tokens = vec![
            Token::new(1, "{"),
            Token::new(7, "key"),
            Token::new(4, ":"),
            Token::new(8, "1000"),
            Token::new(2, ","),
            Token::new(7, "key2 "),
            Token::new(4, ":"),
            Token::new(7, "أَلْحُرُوف ٱلْعَرَبِيَّة😄 😁 😆 value valuluevaluevaluevalue"),
            Token::new(2, ","),
            Token::new(7, "key3"),
            Token::new(4, ":"),
            Token::new(11, "true"),
            Token::new(3, "}")
        ];
        let language: &'static Language = &crate::languages::json::INSTANCE;
        let tps: Vec<u8> = tokens.iter().map(|n| n.tp as u8).collect();
        let mut parser = Parser::new();
        parser.set_language(language.language).unwrap();
        let tree = parser.parse(&tps, None).unwrap();
        let state = EditorState {
            version: 0, language, parser, tokens, tree, font: None, layout: vec![], max_width: 0.0 };
        state
    }
}

fn style(tp: &TokenSpec) -> Color {
    match tp {
        TokenSpec::Constant { semantics, .. } =>
            match semantics {
                ConstantTokenSemantics::Separator => {
                    Color::rgb8(169, 0, 198)
                },
                ConstantTokenSemantics::Delimiter => {
                    Color::rgb8(0, 183, 198)
                },
                ConstantTokenSemantics::Keyword => {
                    Color::rgb8(204, 120, 55)
                },
            },
        TokenSpec::Regex { semantics, .. } =>
            match semantics {
                FreeTokenSemantics::Literal => {
                    Color::rgb8(106, 135, 89)
                },
                FreeTokenSemantics::Unspecified => {
                    Color::rgb8(169, 183, 198)
                },
            }
    }
}

impl Widget<u64> for EditorState {
    fn event(&mut self, ctx: &mut EventCtx, _event: &Event, _data: &mut u64, _env: &Env) {
        ctx.request_focus()
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &u64, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &u64, _data: &u64, _env: &Env) {}

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &u64, _env: &Env) -> Size {
        let mut text = ctx.text();
        if self.font.is_none() {
            self.font = Some(text.new_font_by_name("JetBrains Mono", 14.0).build().unwrap());
        }
        let width = bc.max().width;
        let text = ctx.text();
        self.layout = LayoutParams { state: self, ctx: text, indent: 12.0 }.layout_node(self.tree.root_node(), width).to_lines();
        self.max_width = width;
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &u64, _env: &Env) {
        let layout = &self.layout;
        let mut top = 0.0;
        for line in layout {
            let mut left = line.indent;
            let tokens = &line.tokens;
            let mut ascent: f64 = 0.0;
            let mut descent_and_leading: f64 = 0.0;
            for (_, token) in tokens {
                let metrics = token.layout.line_metric(0).unwrap();
                ascent = ascent.max(metrics.baseline);
                descent_and_leading = descent_and_leading.max(metrics.height - metrics.baseline);
            }
            let height = ascent + descent_and_leading;
            for token in tokens {
                left += token.0;
                let token = &token.1;
                //println!("{}, {}", width, height);
                ctx.draw_text(&token.layout, Point::new(left, top + ascent), &style(&self.language.nodes[token.token.tp as usize].unwrap_as_token()));
                let width = token.layout.width();
                left += width;
            }
            top += height;
        }
    }
}

struct LayoutParams<'a, 'c> {
    state: &'a EditorState,
    ctx: PietText<'c>,
    indent: f64,
}

impl LayoutParams<'_, '_> {
    fn layout_token(&mut self, node: Node, tp: &TokenSpec) -> LayoutResult {
        let token = self.state.tokens[node.start_byte()].clone();
        let layout = self.ctx.new_text_layout(
            self.state.font.as_ref().unwrap(),
            &token.str,
            f64::MAX,
        ).build().unwrap();
        let is_sep = match tp { TokenSpec::Constant {is_separator, .. }=> *is_separator, _ => false };
        let margin = if is_sep { 2.0 } else { 8.0 };
        LayoutResult::Single(TokenLayout {
            token,
            margin_left: margin,
            margin_right: margin,
            is_separator: is_sep,
            layout
        })
    }

    fn layout_node(&mut self, node: Node, max_width: f64) -> LayoutResult {
        let error = node.is_error(); // TODO handle this
        // TODO handle extra nodes
        let nt = node.kind_id();
        match &self.state.language.nodes[nt as usize] {
            NodeSpec::Tree { start, sep, end } => {
                let mut cursor = node.walk();
                let mut children_layout: Vec<(u16, LayoutResult)> = vec![];
                let mut has_child = cursor.goto_first_child();
                let mut is_block = false;
                let mut current_width = 0.0;
                while has_child {
                    let node = cursor.node();
                    let kind = node.kind_id();
                    let child_max_width = if is_block { max_width - self.indent } else { max_width - current_width };
                    let mut layout = self.layout_node(node, child_max_width);
                    match layout {
                        LayoutResult::Block(_) => {
                            is_block = true;
                            // LATER it is possible first item is not a single line after indent is added
                            layout = self.layout_node(node, max_width - self.indent)
                        }
                        _ => {
                            current_width += layout.width();
                            // this happens when the items cannot turns into block but it too long anyway
                            is_block = max_width < current_width;
                        }
                    }
                    children_layout.push((kind, layout));
                    has_child = cursor.goto_next_sibling();
                }
                if is_block {
                    let mut block = Block::new();
                    let mut inside = false;
                    for (role, child) in children_layout {
                        if start.contains(&role) {
                            block.append(child);
                            inside = true;
                        } else if end.contains(&role) {
                            inside = false;
                            block.nl();
                            block.append(child);
                        } else if sep.contains(&role) {
                            block.append(child);
                        } else {
                            block.nl();
                            let mut bl = child.to_block();
                            if inside {
                                bl.indent(self.indent);
                            }
                            block.append_block(bl);
                        }
                    }
                    LayoutResult::Block(block)
                } else {
                    let mut line = Line::new();
                    for (_, child) in children_layout {
                        match child {
                            LayoutResult::Single(a) => {
                                line.push(a);
                            }
                            LayoutResult::Line(b) => {
                                line.append(b);
                            }
                            _ => panic!("not possible")
                        }
                    }
                    LayoutResult::Line(line)
                }
            },
            NodeSpec::Compose => {
                let mut block = Block::new();
                let mut cursor = node.walk();
                let mut has_child = cursor.goto_first_child();
                let mut current_width = 0.0;
                while has_child {
                    let node = cursor.node();
                    let child_max_width = max_width - current_width;
                    let layout = self.layout_node(node, child_max_width);
                    block.append(layout);
                    current_width = block.lines.last_mut().unwrap().width();
                    has_child = cursor.goto_next_sibling();
                }
                block.wrap()
            },
            NodeSpec::Token(tp) => {
                self.layout_token(node, tp)
            },
        }
    }
}
