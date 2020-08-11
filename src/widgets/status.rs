use crate::git;
use crate::state::AppState;
use crate::theme;
use anyhow::Result;
use druid::widget::{CrossAxisAlignment, Flex, Label, List};
use druid::{Data, Lens, LensExt, Widget, WidgetExt};
use git2::{Repository, Status};
use im::{vector, Vector};
use log::{debug, error, info};

pub const ST_NEW: &str = "new";
pub const ST_MODIFIED: &str = "modified";
pub const ST_RENAMED: &str = "renamed";
pub const ST_DELETED: &str = "deleted";
pub const ST_TYPECHANGE: &str = "typechange";

#[derive(Clone, Data, Lens)]
pub struct RepoStatusDetail {
    untracked: Vector<String>,
    unstaged: Vector<(String, String)>,
    staged: Vector<(String, String)>,
    stashed: Vector<String>,
}

impl RepoStatusDetail {
    pub fn new(repo: &Repository) -> Self {
        let statuses = git::get_statuses(repo);
        if let Err(err) = statuses {
            error!("Failed to get status: {}", err);
            return Self::default();
        }
        let mut status = RepoStatusDetail::default();

        for s in statuses.unwrap().iter() {
            let path = s.path().unwrap().to_owned();
            match s.status() {
                Status::WT_NEW => status.untracked.push_back(path),
                Status::WT_MODIFIED => status.unstaged.push_back((ST_MODIFIED.to_owned(), path)),
                Status::WT_RENAMED => status.unstaged.push_back((ST_RENAMED.to_owned(), path)),
                Status::WT_DELETED => status.unstaged.push_back((ST_DELETED.to_owned(), path)),
                Status::WT_TYPECHANGE => {
                    status.unstaged.push_back((ST_TYPECHANGE.to_owned(), path))
                }
                Status::INDEX_NEW => status.staged.push_back((ST_NEW.to_owned(), path)),
                Status::INDEX_MODIFIED => status.staged.push_back((ST_MODIFIED.to_owned(), path)),
                Status::INDEX_RENAMED => status.staged.push_back((ST_RENAMED.to_owned(), path)),
                Status::INDEX_DELETED => status.staged.push_back((ST_DELETED.to_owned(), path)),
                Status::INDEX_TYPECHANGE => {
                    status.staged.push_back((ST_TYPECHANGE.to_owned(), path))
                }
                _ => (),
            }
        }

        status
    }

    pub fn widget() -> impl Widget<AppState> {
        let untracked_header = Flex::row().with_flex_child(
            Label::new("Untracked files").with_text_color(theme::BLUE),
            1.0,
        );
        let untracked_files = Flex::row().with_flex_child(
            List::new(|| Label::new(|item: &String, _env: &_| item.to_owned()))
                .lens(AppState::repo_status.then(RepoStatusDetail::untracked)),
            1.0,
        );

        let unstaged_header = Flex::row().with_flex_child(
            Label::new("Unstaged files").with_text_color(theme::BLUE),
            1.0,
        );
        let unstaged_files = Flex::row().with_flex_child(
            List::new(|| Label::new(|item: &(String, String), _env: &_| item.1.clone()))
                .lens(AppState::repo_status.then(RepoStatusDetail::unstaged)),
            1.0,
        );

        let staged_header = Flex::row()
            .with_flex_child(Label::new("Staged files").with_text_color(theme::BLUE), 1.0);
        let staged_files = Flex::row().with_flex_child(
            List::new(|| Label::new(|item: &(String, String), _env: &_| item.1.clone()))
                .lens(AppState::repo_status.then(RepoStatusDetail::staged)),
            1.0,
        );

        Flex::column()
            .with_child(untracked_header)
            .with_child(untracked_files)
            .with_spacer(24.0)
            .with_child(unstaged_header)
            .with_child(unstaged_files)
            .with_spacer(24.0)
            .with_child(staged_header)
            .with_child(staged_files)
            .cross_axis_alignment(CrossAxisAlignment::Start)
    }
}

impl Default for RepoStatusDetail {
    fn default() -> Self {
        RepoStatusDetail {
            untracked: vector![],
            unstaged: vector![],
            staged: vector![],
            stashed: vector![],
        }
    }
}
