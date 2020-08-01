use crate::state::{AppState, CheatSheetState, Command, KeyMapLevel, L1Node, L2Node};
use crate::theme;
use anyhow::Result;
use druid::widget::{Flex, Label};
use druid::{
    BoxConstraints, Color, Data, Env, Event, EventCtx, KeyCode, LayoutCtx, Lens, LifeCycle,
    LifeCycleCtx, PaintCtx, Rect, RenderContext, Size, UpdateCtx, Widget, WidgetExt, WidgetPod,
};
use druid::{Code, KbKey};
use std::rc::Rc;

fn key_str_to_u8<T: AsRef<str>>(key: T) -> u8 {
    *key.as_ref().as_bytes().get(0).unwrap()
}

pub struct CheatLabel {
    key: WidgetPod<(), Label<()>>,
    desc: WidgetPod<(), Label<()>>,
    origin: (f64, f64),
}

impl CheatLabel {
    pub fn new(key: String, desc: String, origin: (f64, f64)) -> Self {
        let lbl_key = Label::new(key).with_text_color(theme::BLUE);
        let desc = "-> ".to_owned() + &desc;
        let lbl_desc = Label::new(desc).with_text_color(theme::GREEN);

        CheatLabel {
            key: WidgetPod::new(lbl_key),
            desc: WidgetPod::new(lbl_desc),
            origin: origin,
        }
    }
}

pub struct CheatSheet {
    cheat_menu: Vec<CheatLabel>,
    size: Size,
}

impl CheatSheet {
    pub fn new(size: Size) -> Self {
        CheatSheet {
            cheat_menu: vec![],
            size,
        }
    }

    fn update_labels(&mut self, data: &AppState) {
        if data.cheatsheet.is_hidden {
            return;
        }

        self.cheat_menu.clear();

        match data.cheatsheet.current_level {
            KeyMapLevel::L1 => {
                for (key, keymap) in data.cheatsheet.keymap.iter() {
                    self.cheat_menu.push(CheatLabel::new(
                        std::str::from_utf8(&[*key]).unwrap().to_owned(),
                        keymap.name.clone(),
                        (0.0, 0.0),
                    ))
                }
            }
            KeyMapLevel::L2(parent_node) => {
                if let Some(l1_node) = data.cheatsheet.keymap.get(&parent_node) {
                    for (key, keymap) in l1_node.next.iter() {
                        self.cheat_menu.push(CheatLabel::new(
                            std::str::from_utf8(&[*key]).unwrap().to_owned(),
                            keymap.name.clone(),
                            (0.0, 0.0),
                        ))
                    }
                }
            }
        }
    }

    fn reset_menu(&self, data: &mut AppState) {
        data.cheatsheet.current_node = 0;
        data.cheatsheet.current_level = KeyMapLevel::L1;
    }
}

const CHEATSHEET_HEIGHT: f64 = 200.0;
const PADDING_TOP: f64 = 8.0;
const PADDING_LEFT: f64 = 8.0;

impl Widget<AppState> for CheatSheet {
    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        _data: &AppState,
        _env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => {
                ctx.register_for_focus();
            }
            _ => (),
        }

        for cheat in self.cheat_menu.iter_mut() {
            cheat.key.lifecycle(ctx, event, &(), _env);
            cheat.desc.lifecycle(ctx, event, &(), _env);
        }
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, env: &Env) {
        if let Event::WindowConnected = event {
            ctx.request_focus();
        }

        if let Event::Command(cmd) = event {
            if cmd.is(crate::consts::CS_TAKE_FOCUS) {
                ctx.request_focus();
                ctx.set_handled();
                self.reset_menu(data);
            }
        }

        if let Event::KeyUp(key_event) = event {
            let code = key_event.code;
            let key = &key_event.key;

            match code {
                Code::Space => {
                    data.cheatsheet.is_hidden = false;
                }
                Code::Escape => {
                    if !data.cheatsheet.is_hidden {
                        data.cheatsheet.is_hidden = true;
                        self.reset_menu(data);
                    }
                }
                Code::Backspace => {
                    if !data.cheatsheet.is_hidden {
                        if let KeyMapLevel::L2(_) = data.cheatsheet.current_level {
                            self.reset_menu(data);
                        } else {
                            data.cheatsheet.is_hidden = true;
                        }
                    }
                }
                _ => {
                    if let KbKey::Character(c) = key {
                        if let KeyMapLevel::L2(parent_node) = data.cheatsheet.current_level {
                            if let Some(l1_node) = data.cheatsheet.keymap.get(&parent_node) {
                                if let Some(l2_node) = l1_node.next.get(&key_str_to_u8(c)) {
                                    data.cheatsheet.is_hidden = true;

                                    data.fuzzybar.is_hidden = false;
                                    data.fuzzybar.cmd = l2_node.command;

                                    if ctx.is_focused() {
                                        ctx.focus_next();
                                    }
                                }
                            }
                        }

                        if let Some(l1_node) = data.cheatsheet.keymap.get(&key_str_to_u8(c)) {
                            data.cheatsheet.current_node = l1_node.key;
                            data.cheatsheet.current_level = KeyMapLevel::L2(l1_node.key);
                        }
                    }
                }
            }
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old: &AppState, data: &AppState, env: &Env) {
        if data.cheatsheet.is_hidden {
            ctx.request_layout();
            return;
        }

        self.update_labels(data);

        for cheat in self.cheat_menu.iter_mut() {
            cheat.key.update(ctx, &(), env);
            cheat.desc.update(ctx, &(), env);
        }
        ctx.request_layout();
        ctx.request_paint();
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &AppState,
        env: &Env,
    ) -> Size {
        if data.cheatsheet.is_hidden {
            return (0.0, 0.0).into();
        }

        let mut size = bc.max();
        size.width = size.width;
        size.height = CHEATSHEET_HEIGHT;
        self.size = size;

        let max_width = size.width;

        let font_size = env.get(druid::theme::TEXT_SIZE_NORMAL);

        let mut pos_x = 0.0 + PADDING_LEFT;
        let mut pos_y = 0.0 + PADDING_TOP;
        let mut newline = false;

        let child_bc = bc.loosen();
        for cheat in self.cheat_menu.iter_mut() {
            let key_size = cheat.key.layout(ctx, &child_bc, &(), env);
            let desc_size = cheat.desc.layout(ctx, &child_bc, &(), env);

            let next_width = pos_x + key_size.width + desc_size.width;

            if next_width > max_width {
                pos_y = pos_y + font_size + PADDING_TOP;
                pos_x = 0.0 + PADDING_LEFT;
                cheat.origin = (pos_x, pos_y);
                newline = true;
            } else {
                if newline {
                    pos_x = next_width + PADDING_LEFT;
                }
                cheat.origin = (pos_x, pos_y);
                pos_x = next_width + PADDING_LEFT;
            }

            cheat.key.set_layout_rect(
                ctx,
                &(),
                env,
                Rect::from_origin_size(cheat.origin, key_size),
            );

            let desc_origin = (cheat.origin.0 + key_size.width, cheat.origin.1);
            cheat.desc.set_layout_rect(
                ctx,
                &(),
                env,
                Rect::from_origin_size(desc_origin, desc_size),
            );
        }

        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, env: &Env) {
        if data.cheatsheet.is_hidden {
            return;
        }

        let rect = Rect::from(((0.0, 0.0).into(), self.size));
        let base1_color = env.get(theme::BASE_1);
        let bg_color = env.get(theme::BASE_3);
        ctx.blurred_rect(rect, 2.0, &base1_color);
        ctx.fill(rect, &bg_color);

        for cheat in self.cheat_menu.iter_mut() {
            cheat.key.paint(ctx, &(), env);
            cheat.desc.paint(ctx, &(), env);
        }
    }
}
