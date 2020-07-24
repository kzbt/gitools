use anyhow::Result;
use druid::widget::{Container, Flex, Label};
use druid::{AppLauncher, Color, DelegateCtx, Env, Event, Widget, WidgetExt, WindowDesc, WindowId};
use git2::Repository;
use log::info;
use state::AppState;

#[macro_use]
extern crate anyhow;

mod git;
mod state;
mod theme;
mod widget;

fn main() -> Result<()> {
    setup_logger().expect("Failed to setup logger");

    let window = WindowDesc::new(build_root)
        .window_size((1000.0, 800.0))
        .resizable(true)
        .title("Git Tools");

    let args: Vec<String> = std::env::args().collect();
    let repo = Repository::open(&args[1])?;

    let app_state = AppState {
        repo_header: git::get_repo_header(&repo)?,
        cheatsheet: widget::CheatSheetState { is_hidden: true },
    };

    info!("Starting application...");
    AppLauncher::with_window(window)
        .configure_env(|env, state| configure_env(env, state))
        .launch(app_state)
        .expect("Failed to launch app");
    Ok(())
}

fn build_root() -> impl Widget<AppState> {
    let cheatsheet = widget::CheatSheet::new().with_cheat("b".to_owned(), "Branches".to_owned());
    let contents = Flex::column()
        .with_child(git::build_repo_header())
        .with_child(cheatsheet);
    let container = Container::new(contents).background(theme::BASE_3);
    container.debug_paint_layout()
    // container
}

fn configure_env(env: &mut Env, app: &AppState) {
    env.set(theme::FONT_NAME, "Rec Mono Duotone");
    env.set(theme::TEXT_SIZE_NORMAL, 12.0);

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
