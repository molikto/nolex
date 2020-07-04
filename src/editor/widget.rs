use druid::piet::{FontBuilder, Text, TextLayoutBuilder, TextLayout, PietFont, PietText, HitTestTextPosition};
use druid::widget::prelude::*;
use druid::{Point, Color, Rect};
use tree_sitter::{Parser, Node, Tree};

use crate::*;

// TODO partial layout by using layout focus & offset etc. handle scroll ourselves
// TODO reuse text layout for commonly created strs with same attribute?

#[derive(Clone, Debug)]
enum Cursor {
    Point { token: usize, pos: usize }
}

pub struct EditorState {
    version: u64,
    language: &'static Language,
    parser: Parser,
    tokens: Tokens,
    cursor: Cursor,
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
        let cursor = Cursor::Point { token: 0, pos: 0 };
        let state = EditorState {
            version: 0, language, parser,
            tokens, cursor,
            tree,
            font: None, layout: vec![], max_width: 0.0,
        };
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
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut u64, env: &Env) {
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &u64, env: &Env) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &u64, data: &u64, env: &Env) {
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &u64, env: &Env) -> Size {
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
        let cursor = match self.cursor {
            Cursor::Point { token, pos } => (token, pos),
        };
        let mut top = 0.0;
        let mut token_pos: usize = 0;
        for line in layout {
            let mut left = line.indent();
            let tokens = line.tokens();
            let height = line.ascent() + line.descent();
            for token in tokens {
                left += token.0;
                let token = &token.1;
                // draw cursor
                let text_pos = Point::new(left, top + line.ascent());
                let layout = token.layout();
                if token_pos == cursor.0 {
                    let cursor_pos: HitTestTextPosition = layout.hit_test_text_position(cursor.1).unwrap();
                    let x0 = cursor_pos.point.x + text_pos.x;
                    let y = cursor_pos.point.y;
                    let rect = Rect {
                        x0,
                        x1: x0 + 1.0,
                        y0: y - line.ascent(),
                        y1: y + line.descent()
                    };
                    ctx.fill(rect, &Color::grey8(255));
                }
                //println!("{}, {}", width, height);
                ctx.draw_text(layout, text_pos, &style(&self.language.nodes[token.tp() as usize].unwrap_as_token()));
                let width = token.width();
                left += width;
                token_pos += 1;
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
        LayoutResult::Single(TokenLayout::new(
            token,
            margin,
            margin,
            is_sep,
            layout
        ))
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
                    current_width = block.last_width();
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
