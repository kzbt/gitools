use crate::git;
use crate::state::{AppState, Command, FuzzybarState, ListItem};
use crate::theme;
use druid::widget::{Label, List, Painter, Scroll, SizedBox, TextBox};
use druid::{
    BoxConstraints, Code, Color, Data, Env, Event, EventCtx, KbKey, KeyCode, LayoutCtx, Lens,
    LensExt, LifeCycle, LifeCycleCtx, PaintCtx, Rect, RenderContext, Size, UpdateCtx, Widget,
    WidgetExt, WidgetPod,
};
use im::{vector, Vector};
use std::time::{Duration, Instant};

const FUZZYBAR_HEIGHT: f64 = 200.0;
const LABEL_HEIGHT: f64 = 24.0;
const DEBOUNCE_DELTA: Duration = Duration::from_millis(200);

/// Fuzzybar is a fuzzy search bar similar to those provided by those completion
/// frameworks provided by emacs helm. The main components are a querybar which
/// contains the search textbox and a list of items that matches the text.
///
/// A Fuzzybar whenever initiated must be provided with some source data ([`im::Vector<T>`])
/// and a closure that will be invoked on executing a single item
///
/// Fuzzybar also paints its own selection rects and highlights its background.
pub struct Fuzzybar {
    querybar: WidgetPod<AppState, SizedBox<AppState>>,
    matches: WidgetPod<Vector<ListItem>, Scroll<Vector<ListItem>, List<ListItem>>>,
    size: Size,
    ts_since_last_event: Instant,
    selected_idx: usize,
    scrolled: bool,
}

impl Fuzzybar {
    pub fn new() -> Fuzzybar {
        let textbox = TextBox::new()
            .with_placeholder("Search...")
            .lens(AppState::fuzzybar.then(FuzzybarState::query))
            .expand_width();
        let querybar = WidgetPod::new(textbox);

        let scroll = Scroll::new(List::new(|| {
            let painter = Painter::new(|ctx, item: &ListItem, env| {
                let color = if item.selected {
                    env.get(theme::BASE_2)
                } else {
                    env.get(theme::BASE_3)
                };

                let bounds = ctx.size().to_rect();

                ctx.fill(bounds, &color);
            });
            Label::new(|item: &ListItem, _env: &_| item.name.to_owned())
                .padding(3.0)
                .background(painter)
        }))
        .vertical();

        let matches = WidgetPod::new(scroll);

        let size = (0.0, FUZZYBAR_HEIGHT).into();

        Fuzzybar {
            querybar,
            matches,
            size,
            ts_since_last_event: Instant::now(),
            selected_idx: 0,
            scrolled: false,
        }
    }

    fn update_source(&mut self, data: &mut AppState) {
        let source = match data.fuzzybar.cmd {
            Command::BranchCheckout => data.git.all_branches.clone(),
            _ => vector![],
        };

        data.fuzzybar.source = source;
        data.fuzzybar.filter();
        self.selected_idx = 0;
    }

    fn move_selection_up(&mut self, data: &mut AppState) {
        if self.selected_idx == 0 {
            return;
        }

        data.fuzzybar
            .filtered
            .get_mut(self.selected_idx)
            .unwrap()
            .selected = false;
        data.fuzzybar
            .filtered
            .get_mut(self.selected_idx - 1)
            .unwrap()
            .selected = true;
        self.selected_idx -= 1;
    }

    fn scroll_down(&mut self) {
        if self.selected_idx < 8 {
            return;
        }

        let scroll_off = self.matches.widget().offset();
        let next_off = scroll_off + druid::Vec2::new(0.0, LABEL_HEIGHT);
        let delta = next_off - scroll_off;

        let scroll_size = self.matches.widget().child_size();
        let scroll_size = Size::new(scroll_size.width, FUZZYBAR_HEIGHT);
        let scrolled = self.matches.widget_mut().scroll(delta, scroll_size);

        if !self.scrolled {
            self.scrolled = scrolled;
        }
    }

    fn move_selection_down(&mut self, data: &mut AppState) {
        if self.selected_idx + 1 == data.fuzzybar.filtered.len() {
            return;
        }
        data.fuzzybar
            .filtered
            .get_mut(self.selected_idx)
            .unwrap()
            .selected = false;
        data.fuzzybar
            .filtered
            .get_mut(self.selected_idx + 1)
            .unwrap()
            .selected = true;
        self.selected_idx += 1;
    }

    fn scroll_up(&mut self) {
        if self.selected_idx > 12 {
            return;
        }

        let scroll_off = self.matches.widget().offset();
        let next_off = scroll_off - druid::Vec2::new(0.0, LABEL_HEIGHT);
        let delta = next_off - scroll_off;

        let scroll_size = self.matches.widget().child_size();
        let scroll_size = Size::new(scroll_size.width, FUZZYBAR_HEIGHT);
        self.scrolled = self.matches.widget_mut().scroll(delta, scroll_size);
    }

    fn execute_cmd(&mut self, data: &mut AppState) {
        data.fuzzybar.is_hidden = true;
        {
            let selected = data.fuzzybar.filtered.get_mut(self.selected_idx).unwrap();
            git::execute_cmd(&data.repo, data.fuzzybar.cmd, &selected.name);
            data.repo_header = crate::widgets::header::RepoHeader::new(&data.repo).unwrap();
        }
        self.reset_selection(data);
    }

    fn reset_selection(&mut self, data: &mut AppState) {
        let selected = data.fuzzybar.filtered.get_mut(self.selected_idx).unwrap();
        selected.selected = false;

        let first = data.fuzzybar.filtered.get_mut(0).unwrap();
        first.selected = true;

        self.selected_idx = 0;
        self.scrolled = false;
    }
}

impl Widget<AppState> for Fuzzybar {
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppState, env: &Env) {
        match event {
            LifeCycle::WidgetAdded => {
                ctx.register_for_focus();
            }
            _ => (),
        }

        self.querybar.lifecycle(ctx, event, data, env);
        self.matches
            .widget_mut()
            .lifecycle(ctx, event, &data.fuzzybar.filtered, env);
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, env: &Env) {
        if data.fuzzybar.is_hidden {
            return;
        }

        ctx.request_focus();

        if let Event::KeyDown(key_event) = event {
            let code = &key_event.code;
            let mods = &key_event.mods;

            match code {
                Code::Escape => {
                    if !data.fuzzybar.is_hidden {
                        data.fuzzybar.is_hidden = true;
                        self.reset_selection(data);

                        if ctx.is_focused() {
                            ctx.resign_focus();
                            ctx.submit_command(crate::consts::CS_TAKE_FOCUS, None);
                            ctx.set_handled();
                        }
                    }
                }
                Code::Enter => {
                    self.execute_cmd(data);
                    ctx.resign_focus();
                    ctx.submit_command(crate::consts::CS_TAKE_FOCUS, None);
                    ctx.set_handled();
                }
                Code::ControlLeft | Code::ControlRight => {
                    ctx.set_handled();
                }
                Code::KeyJ if mods.ctrl() => {
                    self.move_selection_down(data);
                    self.scroll_down();
                    ctx.set_handled();
                }
                Code::KeyK if mods.ctrl() => {
                    self.move_selection_up(data);
                    self.scroll_up();
                    ctx.set_handled();
                }
                _ if !mods.ctrl() => {
                    self.querybar.widget_mut().event(ctx, event, data, env);

                    let now = Instant::now();
                    let duration_since = now.duration_since(self.ts_since_last_event);
                    if duration_since >= DEBOUNCE_DELTA {
                        self.ts_since_last_event = now;
                        self.update_source(data);
                    }
                    ctx.set_handled();
                }
                _ => ctx.set_handled(),
            }
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old: &AppState, data: &AppState, env: &Env) {
        if data.fuzzybar.is_hidden {
            return;
        }

        self.querybar.update(ctx, data, env);

        if !old.fuzzybar.same(&data.fuzzybar) {
            self.matches.update(ctx, &data.fuzzybar.filtered, env);
        }
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
        size.height = FUZZYBAR_HEIGHT + LABEL_HEIGHT;
        self.size = size;

        let child_bc = bc.loosen();

        let qb_size = self.querybar.layout(ctx, &child_bc, data, env);

        self.querybar
            .set_layout_rect(ctx, data, env, Rect::from_origin_size((0.0, 0.0), qb_size));

        if !self.scrolled {
            let match_size = self
                .matches
                .layout(ctx, &child_bc, &data.fuzzybar.filtered, env);

            self.matches.set_layout_rect(
                ctx,
                &data.fuzzybar.filtered,
                env,
                Rect::from_origin_size((0.0, qb_size.height), match_size),
            );
        }

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
        self.matches.paint(ctx, &data.fuzzybar.filtered, env);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests;
    #[cfg(test)]
    use pretty_assertions::{assert_eq, assert_ne};

    fn test<F>(tf: F)
    where
        F: FnOnce() -> (),
    {
        let has_logger = *tests::HAS_LOGGER;

        if !has_logger {
            tests::setup_test_logger();
        }

        tf();
    }

    #[test]
    fn reset_selection_should_deselect_items_and_scroll() {
        test(|| {
            let (_td, repo) = tests::repo_init();
            let _ = tests::commit(&repo);
            let mut data = tests::state_init(repo);

            let mut fuzzybar = Fuzzybar::new();
            fuzzybar.reset_selection(&mut data);
            assert_eq!(fuzzybar.selected_idx, 0);
            assert_eq!(fuzzybar.scrolled, false);
        })
    }

    #[test]
    fn execute_cmd_branch_checkout() {
        test(|| {
            let (_td, repo) = tests::repo_init();
            let _ = tests::commit(&repo);
            let _ = tests::branch(&repo, "b1");
            let mut data = tests::state_init(repo);
            data.fuzzybar.cmd = Command::BranchCheckout;

            let mut fuzzybar = Fuzzybar::new();
            fuzzybar.selected_idx = data
                .fuzzybar
                .filtered
                .iter()
                .position(|b| b.name == "b1")
                .unwrap();
            fuzzybar.execute_cmd(&mut data);

            let head = data.repo.head().unwrap();
            let branch = head.name().unwrap();
            assert_eq!(branch, "refs/heads/b1");
        })
    }

    #[test]
    fn move_selection_down_should_move_selected() {
        let (_td, repo) = tests::repo_init();
        let _ = tests::commit(&repo);
        let _ = tests::branch(&repo, "b1");
        let mut data = tests::state_init(repo);
        data.fuzzybar.cmd = Command::BranchCheckout;

        let mut fuzzybar = Fuzzybar::new();
        fuzzybar.move_selection_down(&mut data);

        let selected = data
            .fuzzybar
            .filtered
            .get_mut(fuzzybar.selected_idx)
            .unwrap();
        assert_eq!(selected.selected, true);
        assert_eq!(fuzzybar.selected_idx, 1);
    }

    #[test]
    fn move_selection_up_should_move_selected() {
        let (_td, repo) = tests::repo_init();
        let _ = tests::commit(&repo);
        let _ = tests::branch(&repo, "b1");
        let mut data = tests::state_init(repo);
        data.fuzzybar.cmd = Command::BranchCheckout;

        let mut fuzzybar = Fuzzybar::new();
        fuzzybar.move_selection_down(&mut data);
        fuzzybar.move_selection_up(&mut data);

        let selected = data
            .fuzzybar
            .filtered
            .get_mut(fuzzybar.selected_idx)
            .unwrap();
        assert_eq!(selected.selected, true);
        assert_eq!(fuzzybar.selected_idx, 0);
    }
}
