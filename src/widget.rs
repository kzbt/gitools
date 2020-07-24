use crate::state::AppState;
use crate::theme;
use anyhow::Result;
use druid::widget::{Flex, Label};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, KeyCode, LayoutCtx, Lens, LifeCycle, LifeCycleCtx,
    PaintCtx, Rect, Size, UpdateCtx, Widget, WidgetExt, WidgetPod,
};
use keyboard_types::Code;

pub struct Cheat {
    key: WidgetPod<(), Label<()>>,
    desc: WidgetPod<(), Label<()>>,
    origin: (f64, f64),
}

impl Cheat {
    pub fn new(key: String, desc: String, origin: (f64, f64)) -> Self {
        let lbl_key = Label::new(key).with_text_color(theme::BLUE);
        let desc = "-> ".to_owned() + &desc;
        let lbl_desc = Label::new(desc).with_text_color(theme::GREEN);

        Cheat {
            key: WidgetPod::new(lbl_key),
            desc: WidgetPod::new(lbl_desc),
            origin: origin,
        }
    }
}

pub struct CheatSheet {
    cheats: Vec<Cheat>,
}

#[derive(Clone, Data, Lens, Debug)]
pub struct CheatSheetState {
    pub is_hidden: bool,
}

impl CheatSheet {
    pub fn new() -> Self {
        CheatSheet { cheats: vec![] }
    }

    pub fn with_cheat(mut self, key: String, desc: String) -> Self {
        let cheat = Cheat::new(key, desc, (0.0, 0.0));
        self.cheats.push(cheat);
        self
    }
}

impl Widget<AppState> for CheatSheet {
    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        _data: &AppState,
        _env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => ctx.register_for_focus(),
            _ => (),
        }
        for cheat in self.cheats.iter_mut() {
            cheat.key.lifecycle(ctx, event, &(), _env);
            cheat.desc.lifecycle(ctx, event, &(), _env);
        }
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, env: &Env) {
        ctx.request_focus();

        match event {
            Event::KeyUp(key_event) => match key_event.code {
                Code::Space => {
                    data.cheatsheet.is_hidden = false;
                    return;
                }
                Code::Escape => {
                    if !data.cheatsheet.is_hidden {
                        data.cheatsheet.is_hidden = true;
                        return;
                    }
                }
                _ => (),
            },
            _ => (),
        }

        for cheat in self.cheats.iter_mut() {
            // cheat.key.event(ctx, event, &mut (), env);
            // cheat.desc.event(ctx, event, &mut (), env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &AppState, data: &AppState, env: &Env) {
        dbg!(data);

        if data.cheatsheet.is_hidden {
            ctx.request_layout();
            return;
        }

        for cheat in self.cheats.iter_mut() {
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

        let child_bc = bc.loosen();

        for (i, cheat) in self.cheats.iter_mut().enumerate() {
            let key_size = cheat.key.layout(ctx, &child_bc, &(), env);
            // cheat.origin =
            cheat.key.set_layout_rect(
                ctx,
                &(),
                env,
                Rect::from_origin_size(cheat.origin, key_size),
            );

            let desc_size = cheat.desc.layout(ctx, &child_bc, &(), env);
            cheat.desc.set_layout_rect(
                ctx,
                &(),
                env,
                Rect::from_origin_size((20.0, 20.0), desc_size),
            );
        }

        let mut size = bc.max();
        size.height = 100.0;
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, env: &Env) {
        if data.cheatsheet.is_hidden {
            return;
        }

        for cheat in self.cheats.iter_mut() {
            cheat.key.paint(ctx, &(), env);
            cheat.desc.paint(ctx, &(), env);
        }
    }
}
