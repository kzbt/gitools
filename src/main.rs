use anyhow::Result;
use druid::widget::{Container, Flex, Label};
use druid::{
    AppLauncher, Color, DelegateCtx, Env, Event, Selector, Widget, WidgetExt, WindowDesc, WindowId,
};
use git2::Repository;
use im::{vector, Vector};
use log::info;
use state::{AppState, CheatSheetState, Command, Config, FuzzybarState, GitState, KeyMapLevel};
use std::rc::Rc;

#[cfg(test)]
mod tests;

mod git;
mod state;
mod theme;
mod widgets;

const WINDOW_SIZE: (f64, f64) = (1000.0, 800.0);

fn main() -> Result<()> {
    setup_logger().expect("Failed to setup logger");

    let window = WindowDesc::new(build_root)
        .window_size(WINDOW_SIZE)
        .resizable(true)
        .title("Git Tools");

    let args: Vec<String> = std::env::args().collect();
    let repo = Rc::new(Repository::open(&args[1])?);

    let config_str = std::fs::read_to_string("./config.toml").unwrap();
    let config: Config = toml::from_str(&config_str).unwrap();

    let (local, remote) = git::get_branches(&repo);

    let mut all_branches = vector![];
    all_branches.extend(local.iter().cloned());
    all_branches.extend(remote.iter().cloned());

    let header = widgets::header::RepoHeader::new(&repo)?;
    let status = widgets::status::RepoStatusDetail::new(&repo);

    let mut app_state = AppState {
        repo: repo.clone(),
        win_size: WINDOW_SIZE.into(),
        repo_header: header,
        repo_status: status,
        cheatsheet: CheatSheetState {
            is_hidden: true,
            keymap: config.keymap.map,
            current_node: 0,
            current_level: KeyMapLevel::L1,
        },
        fuzzybar: FuzzybarState {
            is_hidden: true,
            cmd: Command::ShowMenu,
            query: "".to_owned(),
            source: all_branches.clone(),
            filtered: vector![],
        },
        git: GitState {
            local_branches: local,
            remote_branches: remote,
            all_branches,
        },
    };

    app_state.fuzzybar.filter();

    info!("Starting application...");
    AppLauncher::with_window(window)
        .configure_env(|env, state| configure_env(env, state))
        .launch(app_state)
        .expect("Failed to launch app");
    Ok(())
}

fn build_root() -> impl Widget<AppState> {
    let fuzzybar = widgets::fuzzybar::Fuzzybar::new();
    let cheatsheet = widgets::cheatsheet::CheatSheet::new(WINDOW_SIZE.into());
    let header = widgets::header::RepoHeader::widget();
    let status = widgets::status::RepoStatusDetail::widget();
    let contents = Flex::column()
        .with_child(header)
        .with_spacer(24.0)
        .with_child(status)
        .with_flex_spacer(1.0)
        .with_child(cheatsheet)
        .with_child(fuzzybar);
    let container = Container::new(contents).background(theme::BASE_3);
    // container.debug_paint_layout().debug_widget_id()
    container
}

fn configure_env(env: &mut Env, app: &AppState) {
    env.set(theme::BASE_3, Color::rgb8(0xfd, 0xf6, 0xe3)); // #fdf6e3
    env.set(theme::BASE_2, Color::rgb8(0xee, 0xe8, 0xd5)); // #eee8d5
    env.set(theme::BASE_1, Color::rgb8(0x93, 0xa1, 0xa1)); // #93a1a1
    env.set(theme::BASE_0, Color::rgb8(0x83, 0x94, 0x96)); // #839496
    env.set(theme::BASE_00, Color::rgb8(0x65, 0x7b, 0x83)); // #657b83
    env.set(theme::BASE_01, Color::rgb8(0x58, 0x6e, 0xe3)); // #586e75
    env.set(theme::BASE_02, Color::rgb8(0x07, 0x36, 0x42)); // #073642
    env.set(theme::BASE_03, Color::rgb8(0x00, 0x2b, 0x36)); // #002b36

    env.set(theme::YELLOW, Color::rgb8(0xb5, 0x89, 0x00)); // #b58900
    env.set(theme::ORANGE, Color::rgb8(0xcb, 0x4b, 0x16)); // #cb4b16
    env.set(theme::RED, Color::rgb8(0xdc, 0x32, 0x2f)); // #dc322f
    env.set(theme::MAGENTA, Color::rgb8(0xd3, 0x36, 0x82)); // #d33682
    env.set(theme::VIOLET, Color::rgb8(0x6c, 0x71, 0xc4)); // #6c71c4
    env.set(theme::BLUE, Color::rgb8(0x26, 0x8b, 0xd2)); // #268bd2
    env.set(theme::CYAN, Color::rgb8(0x2a, 0xa1, 0x98)); // #2aa198
    env.set(theme::GREEN, Color::rgb8(0x85, 0x99, 0x00)); // #859900

    // Overrides
    env.set(druid::theme::FONT_NAME, "Rec Mono Duotone");
    // env.set(druid::theme::FONT_NAME, "RecursiveSansLnr-Regular");
    env.set(druid::theme::TEXT_SIZE_NORMAL, 12.0);
    env.set(druid::theme::BACKGROUND_LIGHT, env.get(theme::BASE_3));
    env.set(druid::theme::LABEL_COLOR, env.get(theme::BASE_00));
    env.set(druid::theme::PRIMARY_LIGHT, env.get(theme::BASE_3));
    env.set(druid::theme::BORDER_DARK, env.get(theme::BASE_3));
}

fn setup_logger() -> Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        // .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

mod consts {
    use druid::Selector;
    pub const CS_TAKE_FOCUS: Selector = Selector::new("gitools.cs.take-focus");
}
