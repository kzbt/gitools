use crate::state::{AppState, Command, FuzzybarState};
use crate::theme;
use druid::widget::{Label, List, Scroll, SizedBox, TextBox};
use druid::{
    BoxConstraints, Code, Color, Data, Env, Event, EventCtx, KbKey, KeyCode, LayoutCtx, Lens,
    LensExt, LifeCycle, LifeCycleCtx, PaintCtx, Rect, RenderContext, Size, UpdateCtx, Widget,
    WidgetExt, WidgetPod,
};
use std::sync::Arc;

pub struct Fuzzybar {
    // querybar: TextBox,
    // matches: WidgetPod<AppState, Scroll<AppState, List<AppState>>>,
    querybar: WidgetPod<AppState, SizedBox<AppState>>,
    matches: WidgetPod<AppState, SizedBox<AppState>>,
    size: Size,
    db: debouncer::Debouncer,
}

impl Fuzzybar {
    pub fn new() -> Fuzzybar {
        let textbox = TextBox::new()
            .with_placeholder("Search...")
            .lens(AppState::fuzzybar.then(FuzzybarState::query))
            .expand_width();
        let querybar = WidgetPod::new(textbox);

        let scroll = Scroll::new(List::new(|| {
            Label::new(|item: &String, env: &Env| item.to_owned())
        }))
        .vertical()
        .lens(AppState::fuzzybar.then(FuzzybarState::filtered))
        .expand_width();

        let matches = WidgetPod::new(scroll);

        let size = (0.0, FUZZYBAR_HEIGHT).into();

        let db = debouncer::Debouncer::new(1);

        Fuzzybar {
            querybar,
            matches,
            size,
            db,
        }
    }

    fn update_source(&self, data: &mut AppState) {
        let source = match data.fuzzybar.cmd {
            Command::BranchCheckout => data.git.all_branches.clone(),
            _ => Arc::new(vec![]),
        };

        data.fuzzybar.source = source;
        data.fuzzybar.filter();
    }
}

const FUZZYBAR_HEIGHT: f64 = 200.0;
const PADDING_TOP: f64 = 8.0;
const PADDING_LEFT: f64 = 8.0;

impl Widget<AppState> for Fuzzybar {
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppState, env: &Env) {
        match event {
            LifeCycle::WidgetAdded => {
                ctx.register_for_focus();
            }
            _ => (),
        }

        self.querybar.lifecycle(ctx, event, data, env);
        self.matches.lifecycle(ctx, event, data, env);
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, env: &Env) {
        if data.fuzzybar.is_hidden {
            return;
        }

        ctx.request_focus();

        self.querybar.widget_mut().event(ctx, event, data, env);

        if let Event::KeyUp(key_event) = event {
            let code = key_event.code;
            let key = &key_event.key;

            match code {
                Code::Escape => {
                    if !data.fuzzybar.is_hidden {
                        data.fuzzybar.is_hidden = true;

                        if ctx.is_focused() {
                            ctx.resign_focus();
                            ctx.submit_command(crate::consts::CS_TAKE_FOCUS, None);
                            ctx.set_handled();
                        }
                    }
                }
                _ => {
                    let db_result = self.db.update(0, true);
                    match db_result {
                        debouncer::DebounceResult::Pressed => {
                            println!("Filtering...");
                            self.update_source(data);
                            self.db.update(0, false);
                            self.db.update(0, false);
                            self.db.update(0, false);
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old: &AppState, data: &AppState, env: &Env) {
        if data.fuzzybar.is_hidden {
            return;
        }

        self.querybar.update(ctx, data, env);
        self.matches.update(ctx, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &AppState,
        env: &Env,
    ) -> Size {
        if data.fuzzybar.is_hidden {
            return (0.0, 0.0).into();
        }

        let mut size = bc.max();
        size.height = FUZZYBAR_HEIGHT;
        self.size = size;

        let child_bc = bc.loosen();

        let qb_size = self.querybar.layout(ctx, &child_bc, data, env);
        let match_size = self.matches.layout(ctx, &child_bc, data, env);

        self.querybar
            .set_layout_rect(ctx, data, env, Rect::from_origin_size((0.0, 0.0), qb_size));

        self.matches.set_layout_rect(
            ctx,
            data,
            env,
            Rect::from_origin_size((0.0, qb_size.height), match_size),
        );

        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, env: &Env) {
        if data.fuzzybar.is_hidden {
            return;
        }

        let rect = Rect::from(((0.0, 0.0).into(), self.size));
        let base1_color = env.get(theme::BASE_1);
        let bg_color = env.get(theme::BASE_3);
        ctx.blurred_rect(rect, 2.0, &base1_color);
        ctx.fill(rect, &bg_color);

        self.querybar.paint(ctx, data, env);
        self.matches.paint(ctx, data, env);
    }
}

mod debouncer {

    pub struct Debouncer {
        patterns: Vec<u8>,
    }

    #[repr(u8)]
    #[derive(PartialEq)]
    pub enum DebounceResult {
        NoChange,
        Pressed,
        Released,
    }

    impl Debouncer {
        pub fn new(no_of_keys: usize) -> Debouncer {
            Debouncer {
                patterns: vec![0; no_of_keys],
            }
        }

        pub fn update(&mut self, key_no: usize, pressed: bool) -> DebounceResult {
            let next: u8 = if pressed { 1 } else { 0 };
            self.patterns[key_no] = self.patterns[key_no] << 1 | next;
            let mut result = DebounceResult::NoChange;
            //debounce following hackadays ultimate debouncing schema
            let mask: u8 = 0b11000111;
            let seen = self.patterns[key_no] & mask;
            if seen == 0b00000111 {
                result = DebounceResult::Pressed;
                self.patterns[key_no] = 0b1111111;
            } else if seen == 0b11000000 {
                result = DebounceResult::Released;
                self.patterns[key_no] = 0b0000000;
            }

            return result;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{DebounceResult, Debouncer};
        #[test]
        fn it_works() {
            let mut db = Debouncer::new(1);
            //activate
            assert!(db.update(0, true) == DebounceResult::NoChange);
            assert!(db.update(0, true) == DebounceResult::NoChange);
            assert!(db.update(0, true) == DebounceResult::Pressed);
            //deactivate
            assert!(db.update(0, false) == DebounceResult::NoChange);
            assert!(db.update(0, false) == DebounceResult::NoChange);
            assert!(db.update(0, false) == DebounceResult::Released);

            //let's do noise.
            assert!(db.update(0, true) == DebounceResult::NoChange);
            assert!(db.update(0, false) == DebounceResult::NoChange);
            assert!(db.update(0, false) == DebounceResult::NoChange);
            assert!(db.update(0, false) == DebounceResult::NoChange);
            assert!(db.update(0, false) == DebounceResult::NoChange);
            assert!(db.update(0, false) == DebounceResult::NoChange);
            assert!(db.update(0, false) == DebounceResult::NoChange);

            assert!(db.update(0, true) == DebounceResult::NoChange);
            assert!(db.update(0, true) == DebounceResult::NoChange);
            assert!(db.update(0, true) == DebounceResult::Pressed);
            assert!(db.update(0, true) == DebounceResult::NoChange);
            assert!(db.update(0, false) == DebounceResult::NoChange);
            assert!(db.update(0, false) == DebounceResult::NoChange);
            assert!(db.update(0, true) == DebounceResult::NoChange);
            assert!(db.update(0, true) == DebounceResult::NoChange);
            assert!(db.update(0, true) == DebounceResult::NoChange);
        }
    }
}
