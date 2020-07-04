use druid::piet::{FontBuilder, Text, TextLayoutBuilder, TextLayout, PietFont, PietText, HitTestTextPosition};
use druid::widget::prelude::*;
use druid::{Point, Color, Rect, Data};
use druid::text::Selection;
use tree_sitter::{Parser, Node, Tree};
use druid::im::vector;

use crate::*;

// TODO partial layout by using layout focus & offset etc. handle scroll ourselves
// TODO reuse text layout for commonly created strs with same attribute?

#[derive(Clone, Debug)]
enum Cursor {
    Point {
        token: usize,
        selection: Selection // TODO blinking!
    }
}

impl PartialEq for Cursor {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Cursor::Point {token: t1, selection: s1}, Cursor::Point { token: t2, selection: s2}) =>
                t1 == t2 && s1.start == s2.start && s1.end == s2.end,
            _ => false
        }
    }
}

impl Eq for Cursor {}


#[derive(Clone)]
pub struct EditorState {
    version: u64,
    tokens: Tokens,
    cursor: Cursor
}

impl EditorState {
    pub fn new() -> EditorState {
        let tokens = vector![
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
        let cursor = Cursor::Point { token: 0, selection: Selection { start : 0, end : 0 } };
        EditorState {
            version: 0,
            tokens, cursor
        }
    }
}

impl Data for EditorState {
    fn same(&self, other: &Self) -> bool {
        self.version == other.version && self.cursor == other.cursor
    }
}

pub struct EditorWidget {
    language: &'static Language,
    parser: Parser,
    tree: Option<Tree>,
    font: Option<PietFont>,
    max_width: f64,
    layout: Vec<Line>,
}

impl EditorWidget {
    pub fn new() -> EditorWidget {
        let language: &'static Language = &crate::languages::json::INSTANCE;
        let mut parser = Parser::new();
        parser.set_language(language.language).unwrap();
        let state = EditorWidget {
            language, parser,
            tree: None,
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

impl Widget<EditorState> for EditorWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut EditorState, env: &Env) {
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &EditorState, env: &Env) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &EditorState, data: &EditorState, env: &Env) {
        ctx.request_paint();
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &EditorState, env: &Env) -> Size {
        let mut text = ctx.text();
        if self.tree.is_none() {
            let tps: Vec<u8> = data.tokens.iter().map(|n| n.tp as u8).collect();
            self.tree = Some(self.parser.parse(&tps, None).unwrap());
        }
        if self.font.is_none() {
            self.font = Some(text.new_font_by_name("JetBrains Mono", 14.0).build().unwrap());
        }
        let width = bc.max().width;
        let text = ctx.text();
        self.layout = LayoutParams {
            state: data,
            widget: self,
            ctx: text, indent: 12.0
        }.layout_node(self.tree.as_ref().unwrap().root_node(), width).to_lines();
        self.max_width = width;
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &EditorState, env: &Env) {
        let layout = &self.layout;
        let cursor = match data.cursor {
            Cursor::Point { token, selection } => (token, selection.end),
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






struct LayoutParams<'a, 'b, 'c> {
    state: &'a EditorState,
    widget: &'b EditorWidget,
    ctx: PietText<'c>,
    indent: f64,
}

impl LayoutParams<'_, '_, '_> {
    fn layout_token(&mut self, node: Node, tp: &TokenSpec) -> LayoutResult {
        let token = self.state.tokens[node.start_byte()].clone();
        let layout = self.ctx.new_text_layout(
            self.widget.font.as_ref().unwrap(),
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
        match &self.widget.language.nodes[nt as usize] {
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
