use druid::piet::{FontBuilder, Text, TextLayoutBuilder, TextLayout, PietFont, PietTextLayout, PietText};
use druid::widget::prelude::*;
use druid::{Point};
use tree_sitter::{Parser, Node, Tree};

use crate::*;
use crate::languages::json;
use std::ops::DerefMut;

// TODO partial layout by using layout focus & offset etc. handle scroll ourselves
// TODO reuse text layout for commonly created strs with same attribute?

struct TokenLayout {
    token: Token,
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
    tokens: Vec<TokenLayout>,
}


impl Line {
    fn width(&self) -> f64 {
        let remaining: f64 = self.tokens.iter().map(|n| n.width()).sum();
        self.indent + remaining
    }
}

// should always be multiple line
struct Block {
    lines: Vec<Line>
}

impl Block {
    fn width(&self) -> f64 {
        self.lines.iter().map(|n| n.width()).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
    }

    fn new() -> Block {
        Block { lines: vec![Line { indent: 0.0, tokens: vec![] }] }
    }

    fn wrap(mut self) -> LayoutResult {
        if self.lines.len() == 1 {
            LayoutResult::Line(self.lines.remove(0))
        } else {
            LayoutResult::Block(self)
        }
    }

    fn append_block(&mut self, mut b: Block) {
        let mut last = &mut self.lines.last_mut().unwrap();
        if last.tokens.is_empty() {
            self.lines.remove(self.lines.len() - 1);
        }
        self.lines.append(&mut b.lines);
    }
    fn append(&mut self, res: LayoutResult) {
        let mut last = &mut self.lines.last_mut().unwrap();
        match res {
            LayoutResult::Single(a) => {
                last.tokens.push(a)
            }
            LayoutResult::Line(mut l) => last.tokens.append(&mut l.tokens),
            LayoutResult::Block(mut b) => {
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
        self.lines.push(Line { indent: 0.0, tokens: vec![] })
    }
}

enum LayoutResult {
    Single(TokenLayout),
    Line(Line),
    Block(Block),
}

impl LayoutResult {
    fn width(&self) -> f64 {
        match self {
            LayoutResult::Single(t) => t.width(),
            LayoutResult::Line(l) => l.width(),
            LayoutResult::Block(b) => b.width()
        }
    }

    fn to_lines(self) -> Vec<Line> {
        match self {
            LayoutResult::Single(t) => vec![Line { indent: 0.0, tokens: vec![t] }],
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
    tokens: Tokens,
    parser: Parser,
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
            Token::new(10, "1000"),
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
        let tps: Vec<u8> = tokens.iter().map(|n| n.tp as u8).collect();
        let mut parser = crate::languages::json::new_parser();
        let tree = parser.parse(&tps, None).unwrap();
        let state = EditorState { version: 0, tokens, parser, tree, font: None, layout: vec![], max_width: 0.0 };
        state
    }
}

impl Widget<u64> for EditorState {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut u64, env: &Env) {
        ctx.request_focus()
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &u64, env: &Env) {}

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &u64, data: &u64, env: &Env) {}

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &u64, env: &Env) -> Size {
        let mut text = ctx.text();
        if self.font.is_none() {
            self.font = Some(text.new_font_by_name("JetBrains Mono Regular", 14.0).build().unwrap());
        }
        let width = bc.max().width;
        // TODO this layout is trivial, we just layout all stuff in a line, without even spaces!
        let mut text = ctx.text();
        self.layout = LayoutParams { state: self, ctx: text, indent: 12.0 }.layout_node(self.tree.root_node(), width).to_lines();
        self.max_width = width;
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &u64, env: &Env) {
        let layout = &self.layout;
        let mut top = 0.0;
        for line in layout {
            let mut left = line.indent;
            let tokens = &line.tokens;
            let mut ascent: f64 = 0.0;
            let mut descent_and_leading: f64 = 0.0;
            for token in tokens {
                let metrics = token.layout.line_metric(0).unwrap();
                ascent = ascent.max(metrics.baseline);
                descent_and_leading = descent_and_leading.max(metrics.height - metrics.baseline);
            }
            let height = ascent + descent_and_leading;
            for token in tokens {
                let width = token.layout.width();
                //println!("{}, {}", width, height);
                ctx.draw_text(&token.layout, Point::new(left, top + ascent), &json::style(&token.token));
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
    fn layout_token(&mut self, node: Node) -> LayoutResult {
        let token = self.state.tokens[node.start_byte()].clone();
        let layout = self.ctx.new_text_layout(
            self.state.font.as_ref().unwrap(),
            &token.str,
            f64::MAX,
        ).build().unwrap();
        LayoutResult::Single(TokenLayout { token, layout })
    }

    fn layout_node(&mut self, node: Node, max_width: f64) -> LayoutResult {
        let error = node.is_error(); // TODO handle this
        let tp = node.kind_id();
        if json::is_tree(tp) {
            let mut cursor = node.walk();
            let mut children_layout: Vec<(TokenRole, LayoutResult)> = vec![];
            let mut has_child = cursor.goto_first_child();
            let mut is_block = false;
            let mut current_width = 0.0;
            while has_child {
                let node = cursor.node();
                let role = json::token_role(tp, node.kind_id());
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
                children_layout.push((role, layout));
                has_child = cursor.goto_next_sibling();
            }
            if is_block {
                let mut block = Block::new();
                let mut inside = false;
                for (role, mut child) in children_layout {
                    match role {
                        TokenRole::TreeStart => {
                            block.append(child);
                            inside = true;
                        }
                        TokenRole::TreeEnd => {
                            inside = false;
                            block.nl();
                            block.append(child);
                        }
                        TokenRole::Sep => {
                            block.append(child);
                        }
                        _ => {
                            block.nl();
                            let mut bl = child.to_block();
                            bl.indent(self.indent);
                            block.append_block(bl);
                        }
                    }
                }
                LayoutResult::Block(block)
            } else {
                let mut tokens: Vec<TokenLayout> = vec![];
                for (_, child) in children_layout {
                    match child {
                        LayoutResult::Single(a) => {
                            tokens.push(a);
                        }
                        LayoutResult::Line(mut b) => {
                            tokens.append(&mut b.tokens);
                        }
                        _ => panic!("not possible")
                    }
                }
                LayoutResult::Line(Line { indent: 0.0, tokens })
            }
        } else if json::is_token(tp) {
            self.layout_token(node)
        } else {
            let mut block = Block::new();
            let mut cursor = node.walk();
            let mut has_child = cursor.goto_first_child();
            let mut current_width = 0.0;
            while has_child {
                let node = cursor.node();
                let role = json::token_role(tp, node.kind_id());
                let child_max_width = max_width - current_width;
                let mut layout = self.layout_node(node, child_max_width);
                block.append(layout);
                current_width = block.lines.last().unwrap().width();
                has_child = cursor.goto_next_sibling();
            }
            block.wrap()
        }
    }
}
