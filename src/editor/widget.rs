use druid::piet::{FontBuilder, Text, TextLayoutBuilder, TextLayout, PietFont, PietText};
use druid::widget::prelude::*;
use druid::{Point, Color, Rect};
use druid::text::{Selection, BasicTextInput, TextInput, EditAction, EditableText, offset_for_delete_backwards, Movement, movement};
use tree_sitter::{Parser, Node, Tree, InputEdit};
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


pub struct EditorState {
    version: u64,
    language: &'static Language,
    tokens: Tokens,
    cursor: Cursor,
    parser: Parser,
    tree: Tree
}

const tree_sitter_point_zero: tree_sitter::Point = tree_sitter::Point { row: 0, column: 0 };
const tree_sitter_point_one: tree_sitter::Point = tree_sitter::Point { row: 0, column: 1 };

impl EditorState {
    pub fn new() -> EditorState {
        let tokens = vector![
            Token::new(1, "{"),
            Token::new(7, "key2"),
            Token::new(4, ":"),
            Token::new(7, "valuevaluevalueأَلْحُرُوف ٱلْعَرَبِيَّة😄😁😆 value valul"),
            Token::new(2, ","),
            Token::new(7, "key"),
            Token::new(4, ":"),
            Token::new(5, "["),
            Token::new(7, "key"),
            Token::new(2, ","),
            Token::new(7, "key"),
            Token::new(2, ","),
            Token::new(7, "key"),
            Token::new(2, ","),
            Token::new(7, "key"),
            Token::new(2, ","),
            Token::new(7, "key"),
            Token::new(2, ","),
            Token::new(7, "key"),
            Token::new(2, ","),
            Token::new(7, "key"),
            Token::new(2, ","),
            Token::new(7, "key"),
            Token::new(2, ","),
            Token::new(7, "key"),
            Token::new(2, ","),
            Token::new(7, "key"),
            Token::new(2, ","),
            Token::new(7, "key"),
            Token::new(6, "]"),
            Token::new(2, ","),
            Token::new(7, "key"),
            Token::new(4, ":"),
            Token::new(8, "1000"),
            Token::new(2, ","),
            Token::new(7, "key3"),
            Token::new(4, ":"),
            Token::new(11, "true"),
            Token::new(3, "}")
        ];
        let cursor = Cursor::Point { token: 0, selection: Selection { start : 0, end : 0 } };
        let language: &'static Language = &crate::languages::json::INSTANCE;

        let mut parser = Parser::new();
        parser.set_language(language.language()).unwrap();
        // TODO bad
        let tps: Vec<u8> = tokens.iter().map(|n| n.tp as u8).collect();
        let tree = parser.parse(&tps, None).unwrap();
        EditorState {
            version: 0,
            language,
            tokens, cursor, tree, parser
        }
    }

    fn reparse(&mut self, start: usize, end: usize, new_end: usize) {
        self.version += 1;
        self.tree.edit(&InputEdit {
            start_byte: start,
            old_end_byte: end,
            new_end_byte: new_end,
            start_position: tree_sitter_point_zero,
            old_end_position: tree_sitter_point_one,
            new_end_position: tree_sitter_point_one
        });
        // TODO bad
        let tps: Vec<u8> = self.tokens.iter().map(|n| n.tp as u8).collect();
        self.tree = self.parser.parse(tps, Some(&self.tree)).unwrap();
        if self.tokens.is_empty() {
            self.tokens.push_back(Token::new(self.language.lex_error(), ""));
            self.reparse(0, 0, 1)
        } else if self.language.node(self.tokens[0].tp).as_token().is_separator() {
            self.tokens.push_front(Token::new(self.language.lex_error(), ""));
            self.reparse(0, 0, 1)
        } else if self.language.node(self.tokens.last().unwrap().tp).as_token().is_separator() {
            let len = self.tokens.len();
            self.tokens.push_back(Token::new(self.language.lex_error(), ""));
            self.reparse(len, len, len + 1);
        }
    }

    fn lex_sync_then_sit(&mut self, t: usize) {
        let token = &mut self.tokens[t];
        let spec = self.language.node(token.tp).as_token();
        if token.str.is_empty() {
            if !spec.can_empty() {
                self.move_selection(Movement::Left, false);
                self.tokens.remove(t);
                self.reparse(t, t + 1, t);
            }
        } else {
            if spec.is_lex_error() {
                if let Some(tp) = self.language.try_lex(&token.str) {
                    token.tp = tp;
                    self.reparse(t, t + 1, t + 1);
                }
            } else {
                if !spec.accept(&token.str) {
                    token.tp = self.language.lex_error();
                    self.reparse(t, t + 1, t + 1);
                }
            }
        }
    }

    fn insert(&mut self, new: &str) {
        match &mut self.cursor {
            Cursor::Point { token, selection } => {
                let token = *token;
                let text = &mut self.tokens[token].str;
                *selection = selection.constrain_to(text);
                text.edit(selection.range(), new);
                *selection = Selection::caret(selection.min() + new.len());
                self.lex_sync_then_sit(token);
            },
        }
    }

    fn delete_backward(&mut self) {
        match &mut self.cursor {
            Cursor::Point { token, selection } => {
                let token = *token;
                let text = &mut self.tokens[token].str;
                if text.is_empty() {
                    self.move_selection(Movement::Left, false);
                    self.tokens.remove(token);
                    self.reparse(token, token + 1, token);
                } else {
                    let to = if selection.is_caret() {
                        let cursor = selection.end;
                        let new_cursor = offset_for_delete_backwards(&selection, text);
                        text.edit(new_cursor..cursor, "");
                        new_cursor
                    } else {
                        text.edit(selection.range(), "");
                        selection.min()
                    };
                    match text.cursor(to) {
                        Some(_) => *selection = Selection::caret(to),
                        None => panic!()
                    }
                    self.lex_sync_then_sit(token);
                }
            },
        }
    }

    fn delete_forward(&mut self) {
        match &mut self.cursor {
            Cursor::Point { token, selection } => {
                let token = *token;
                let text = &mut self.tokens[token].str;
                if text.is_empty() {

                    self.tokens.remove(token);
                    self.reparse(token, token + 1, token);
                } else {
                    if selection.is_caret() {
                        // Never touch the characters before the cursor.
                        if text.next_grapheme_offset(selection.end).is_some() {
                            self.move_selection(Movement::Right, false);
                            self.delete_backward();
                        }
                    } else {
                        self.delete_backward();
                    }
                }
            },
        }
    }

    /// Edit a selection using a `Movement`.
    fn move_selection(&mut self, mvmnt: Movement, modify: bool) {
        match &mut self.cursor {
            Cursor::Point { token, selection } => {
                let mut index = *token;
                let text = &self.tokens[index].str;
                // let self_separator = self.language.node(self.tokens[index].tp).as_token().is_separator();
                // This movement function should ensure all movements are legit.
                // If they aren't, that's a problem with the movement function.
                match mvmnt {
                    Movement::Left if selection.end == 0 => {
                        if index > 0 {
                            index -= 1;
                            let token_next = self.tokens[index].tp;
                            if self.language.node(token_next).as_token().is_separator() {
                                if index > 0 {
                                    index -= 1;
                                }
                            }
                            self.cursor = Cursor::Point { token: index as usize, selection: Selection::caret(self.tokens[index].str.len()) }
                        }
                    },
                    Movement::Right if selection.end == text.len() => {
                        let mut index = *token;
                        if index < self.tokens.len() - 1 {
                            index += 1;
                            let token_next = self.tokens[index].tp;
                            if self.language.node(token_next).as_token().is_separator() {
                                if index < self.tokens.len() - 1 {
                                    index += 1;
                                }
                            }
                            self.cursor = Cursor::Point { token: index, selection: Selection::caret(0) }
                        }
                    },
                    _ => {
                        *selection = movement(mvmnt, *selection, text, modify);
                    }
                }
            },
        }
    }

    fn do_edit_action(&mut self, edit_action: EditAction) {
        match edit_action {
            EditAction::Insert(chars)  => {
                self.insert(&chars);
            },
            // | EditAction::Paste(chars)
            EditAction::Backspace => {
                self.delete_backward();
            },
            EditAction::Delete => {
                self.delete_forward();
            },
            EditAction::Move(movement) => self.move_selection(movement, false),
            _ => {}
            //EditAction::ModifySelection(movement) => self.move_selection(movement, true),
            //EditAction::SelectAll => selection.all(),
            // EditAction::Click(action) => {
            //     if action.mods.shift() {
            //         self.selection.end = action.column;
            //     } else {
            //         self.caret_to(text, action.column);
            //     }
            // }
            //EditAction::Drag(action) => self.selection.end = action.column,
        }
    }
}

pub struct EditorWidget {
    basic: BasicTextInput,
    font: Option<PietFont>,
    max_width: f64,

    data: Option<EditorState>,
    layout: Vec<Line>,
}

impl EditorWidget {
    pub fn new() -> EditorWidget {
        let state = EditorWidget {
            basic: BasicTextInput::new(), data: None,
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
                RegexTokenSemantics::Literal => {
                    Color::rgb8(106, 135, 89)
                },
                RegexTokenSemantics::Unspecified => {
                    Color::rgb8(169, 183, 198)
                },
                RegexTokenSemantics::LexingError => {
                    Color::rgb8(255, 0, 0)
                }
            }
    }
}

impl EditorWidget {
    fn data(&self) -> &EditorState {
        self.data.as_ref().unwrap()
    }
}
impl Widget<u64> for EditorWidget {
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _: &u64, env: &Env) {
        match event {
            LifeCycle::WidgetAdded => {
                ctx.register_for_focus()
            },
            // an open question: should we be able to schedule timers here?
            // LifeCycle::FocusChanged(true) => ctx.submit_command(RESET_BLINK, ctx.widget_id()),
            _ => (),
        }

        if self.data.is_none() {
            self.data = Some(EditorState::new());
        }
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut u64, env: &Env) {
        match event {
            Event::KeyDown(key_event) => {
                let edit_action = self.basic.handle_event(key_event);
                if let Some(edit_action) = edit_action {
                    self.data.as_mut().unwrap().do_edit_action(edit_action);
                    ctx.request_paint();
                    ctx.request_layout();
                }
            },
            _ => {

            }
        }
        if !ctx.has_focus() {
            ctx.request_focus();
        }

        *data = self.data.as_ref().unwrap().version;
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &u64, _: &u64, env: &Env) {
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _: &u64, env: &Env) -> Size {
        let mut text = ctx.text();
        if self.font.is_none() {
            self.font = Some(text.new_font_by_name("JetBrains Mono", 14.0).build().unwrap());
        }
        let width = bc.max().width;
        let text = ctx.text();
        let data = self.data();
        self.layout = LayoutParams {
            tokens: &data.tokens,
            language: &data.language,
            widget: self,
            ctx: text, indent: 12.0
        }.layout(&data.tree, width);
        self.max_width = width;
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _: &u64, env: &Env) {
        let layout = &self.layout;
        let data = self.data();
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
                // TODO fat cursor for empty token
                if token_pos == cursor.0 {
                    let cursor_pos = if token.is_empty() {
                        0.0
                    } else {
                        let pos: Point = layout.hit_test_text_position(cursor.1).unwrap().point;
                        pos.x
                    };
                    let x0 = cursor_pos + text_pos.x;
                    let y = text_pos.y;
                    let rect = Rect {
                        x0,
                        x1: x0 + 1.0,
                        y0: y - line.ascent(),
                        y1: y + line.descent()
                    };
                    ctx.fill(rect, &Color::grey8(255));
                }
                // let rect = Rect {
                //     x0: left,
                //     x1: left + layout.width(),
                //     y0: top,
                //     y1: top + line.descent() + line.ascent()
                // };
                // ctx.fill(rect, &Color::grey8(255));
                ctx.draw_text(layout, text_pos, &style(&data.language.node(token.tp()).as_token()));
                let width = token.width();
                left += width;
                token_pos += 1;
            }
            top += height;
        }
    }
}






// TODO maybe font caching should be done somewhere, so widget is not a parameter anymore
struct LayoutParams<'a, 'b, 'c> {
    tokens: &'a Tokens,
    language: &'static Language,
    widget: &'b EditorWidget,
    ctx: PietText<'c>,
    indent: f64,
}

impl LayoutParams<'_, '_, '_> {
    fn layout_token(&mut self, node: Node, tp: &TokenSpec) -> LayoutResult {
        let token = self.tokens[node.start_byte()].clone();
        let layout = self.ctx.new_text_layout(
            self.widget.font.as_ref().unwrap(),
            if token.str.is_empty() { "*" } else { &token.str },
            f64::MAX,
        ).build().unwrap();
        let is_sep = tp.is_separator();
        let margin = if is_sep { 2.0 } else { 8.0 };
        LayoutResult::Single(TokenLayout::new(
            token,
            margin,
            margin,
            is_sep,
            layout
        ))
    }

    fn layout(&mut self, tree: &Tree, max_width: f64) -> Vec<Line> {
        self.layout_node(tree.root_node(), 0, max_width, max_width).to_lines()
    }

    fn layout_node(&mut self, node: Node, depth: i32, max_width_first: f64, max_width_remaining: f64) -> LayoutResult {
        let error = node.is_error(); // TODO handle this
        let nt = node.kind_id();
        // println!("layouting node {}, {}, with {}, {}", nt, depth, max_width_first, max_width_remaining);
        if node.is_missing() { panic!("missing node with type {}", nt) }; // we don't know what to do yet.
        if node.is_extra() && nt != 65535 { panic!("extra node is not handled {}", nt) }; // we don't know what to do yet.
        // TODO handle extra nodes
        match &self.language.node(nt) {
            NodeSpec::Tree { start, sep, end } => {
                let mut cursor = node.walk();
                let mut children_layout: Vec<(u16, LayoutResult)> = vec![];
                let mut has_child = cursor.goto_first_child();
                let mut is_block = false;
                let mut current_width = 0.0;
                while has_child {
                    let node = cursor.node();
                    let kind = node.kind_id();
                    let (c_first, c_remaining) = if is_block {
                        let width = max_width_remaining - self.indent;
                        (width, width)
                    } else {
                        (max_width_first - current_width, max_width_remaining - self.indent)
                    };

                    let mut layout = self.layout_node(node, depth + 1, c_first, c_remaining);
                    let layout = match &mut layout {
                        LayoutResult::Block(_) => {
                            is_block = true;
                            // LATER it is possible first item is not a single line after indent is added
                            // TODO this causes massive relayout sometimes?
                            let width = max_width_remaining - self.indent;
                            self.layout_node(node,  depth + 1, width, width)
                        }
                        mut ll => {
                            if !is_block {
                                let layout_width = ll.width();
                                // println!("{}, {}", current_width, layout_width);
                                // this happens when the items cannot turns into block but it too long anyway
                                let t1 : Option<&TokenLayout> = children_layout.last().and_then(|n| match &n.1 {
                                    LayoutResult::Line(l) => l.last(),
                                    LayoutResult::Single(s) => Some(&s),
                                    _ => panic!()
                                });
                                let t2 = match ll {
                                    LayoutResult::Line(l) => l.last().unwrap(),
                                    LayoutResult::Single(s) => s,
                                    _ => panic!()
                                };
                                let added_width = layout_width + Line::merge_margin(t1, t2);
                                current_width += added_width;
                                is_block = c_first < added_width;
                                // if is_block {
                                //     println!("turned into block, {}, {}", c_first, current_width);
                                // }
                            }
                            layout
                        }
                    };
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
                            block.nl(0.0);
                            block.append(child);
                        } else if sep.contains(&role) {
                            block.append(child);
                        } else {
                            block.nl(self.indent);
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
            NodeSpec::Token(tp) => {
                self.layout_token(node, tp)
            },
            _ => {
                let mut block = Block::new();
                let mut cursor = node.walk();
                let mut has_child = cursor.goto_first_child();
                let mut current_width = 0.0;
                while has_child {
                    let node = cursor.node();
                    let max_width = if block.single_line() { max_width_first } else { max_width_remaining };
                    let child_max_width = max_width - current_width;
                    let layout = self.layout_node(node, depth + 1, child_max_width, max_width_remaining);
                    block.append(layout);
                    current_width = block.last_width();
                    has_child = cursor.goto_next_sibling();
                }
                block.wrap()
            },
        }
    }
}
