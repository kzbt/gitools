use crate::git;
use crate::state::{
    AppState, CheatSheetState, Command, Config, FuzzybarState, GitState, KeyMapLevel,
};
use anyhow::Result;
use git2::{Branch, Oid, Repository};
use im::vector;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::ptr;
use std::rc::Rc;
#[cfg(test)]
use tempfile::TempDir;

macro_rules! res {
    ($e:expr) => {
        match $e {
            Ok(e) => e,
            Err(e) => panic!("{} failed with {}", stringify!($e), e),
        }
    };
}

pub fn repo_init() -> (TempDir, Repository) {
    let td = TempDir::new().unwrap();
    let repo = Repository::init(td.path()).unwrap();
    {
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "name").unwrap();
        config.set_str("user.email", "email").unwrap();
        let mut index = repo.index().unwrap();
        let id = index.write_tree().unwrap();

        let tree = repo.find_tree(id).unwrap();
        let sig = repo.signature().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();
    }
    (td, repo)
}

pub fn state_init(repo: Repository) -> AppState {
    let config_str = std::fs::read_to_string("./config.toml").unwrap();

    let config: Config = toml::from_str(&config_str).unwrap();

    let (local, remote) = git::get_branches(&repo);

    let mut all_branches = vector![];
    all_branches.extend(local.iter().cloned());
    all_branches.extend(remote.iter().cloned());

    let repo = Rc::new(repo);

    let mut app_state = AppState {
        repo: repo.clone(),
        win_size: crate::WINDOW_SIZE.into(),
        repo_header: res!(git::get_repo_header(&repo)),
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

    app_state
}

pub fn commit(repo: &Repository) -> (Oid, Oid) {
    let mut index = res!(repo.index());
    let root = repo.path().parent().unwrap();
    res!(File::create(&root.join("foo")));
    res!(index.add_path(Path::new("foo")));

    let tree_id = res!(index.write_tree());
    let tree = res!(repo.find_tree(tree_id));
    let sig = res!(repo.signature());
    let head_id = res!(repo.refname_to_id("HEAD"));
    let parent = res!(repo.find_commit(head_id));
    let commit = res!(repo.commit(Some("HEAD"), &sig, &sig, "commit", &tree, &[&parent]));
    (commit, tree_id)
}

pub fn branch<'a>(repo: &'a Repository, branch_name: &str) -> Branch<'a> {
    let head_id = res!(repo.refname_to_id("HEAD"));
    let target = res!(repo.find_commit(head_id));
    res!(repo.branch(branch_name, &target, false))
}

lazy_static::lazy_static! {
    pub static ref HAS_LOGGER: bool = setup_test_logger();
}

pub fn setup_test_logger() -> bool {
    println!("Setting up test logger...");
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
        .chain(fern::Output::call(|record| println!("{}", record.args())))
        .apply()
        .unwrap();
    true
}
