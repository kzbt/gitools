use crate::git;
use crate::state::{AppState, Command};
use crate::theme;
use anyhow::{anyhow, Result};
use druid::widget::{Align, Container, CrossAxisAlignment, Flex, Label, SizedBox};
use druid::{Data, Env, Widget, WidgetExt};
use git2::{BranchType, DescribeFormatOptions, DescribeOptions, Reference, Repository};
use log::{debug, info};

#[derive(Clone, Data, Debug)]
pub struct RepoHeader {
    local_head: (String, String),
    remote_head: (String, String),
    tag: String,
}

pub fn build_repo_header() -> impl Widget<AppState> {
    let lbl_head = Label::new("Head:")
        .with_text_color(theme::BASE_00)
        .fix_width(80.0);
    let lbl_head_name = Label::dynamic(|app: &AppState, _| app.repo_header.local_head.0.clone())
        .with_text_color(theme::CYAN);
    let lbl_head_msg = Label::dynamic(|app: &AppState, _| app.repo_header.local_head.1.clone())
        .with_text_color(theme::BASE_00);

    let row_head = Flex::row()
        .with_child(lbl_head)
        .with_flex_child(lbl_head_name, 1.0)
        .with_flex_child(lbl_head_msg, 2.0);

    let lbl_ups = Label::new("Remote:")
        .with_text_color(theme::BASE_00)
        .fix_width(80.0);
    let lbl_ups_name = Label::dynamic(|app: &AppState, _| app.repo_header.remote_head.0.clone());
    let lbl_ups_msg = Label::dynamic(|app: &AppState, _| app.repo_header.remote_head.1.clone());

    let row_ups = Flex::row()
        .with_child(lbl_ups)
        .with_flex_child(lbl_ups_name.with_text_color(theme::GREEN), 1.0)
        .with_flex_child(lbl_ups_msg.with_text_color(theme::BASE_00), 0.0);

    let lbl_tag = Label::new("Tag:")
        .with_text_color(theme::BASE_00)
        .fix_width(80.0);
    let lbl_tag_tag = Label::dynamic(|app: &AppState, _| app.repo_header.tag.clone());

    let row_tag = Flex::row()
        .with_child(lbl_tag)
        .with_child(lbl_tag_tag.with_text_color(theme::YELLOW));

    let layout = Flex::column()
        .with_child(row_head)
        .with_child(row_ups)
        .with_child(row_tag)
        .cross_axis_alignment(CrossAxisAlignment::Start);
    layout
}

pub fn get_repo_header(repo: &Repository) -> Result<RepoHeader> {
    let tag = git::get_latest_tag(repo);

    let head = repo.head()?;
    let head_commit_msg = git::get_commit_from_ref(repo, &head)?;
    let head_full = head.name().unwrap_or_default();
    let head_short = head.shorthand().unwrap_or_default();

    if let Ok(upstream) = repo.branch_upstream_name(head_full) {
        let upstream = upstream.as_str().unwrap_or_default();
        let upstream_ref = repo.find_reference(upstream)?;
        let upstream_short = upstream_ref.shorthand().unwrap_or_default();
        let upstream_commit_msg = git::get_commit_from_ref(repo, &upstream_ref)?;

        return Ok(RepoHeader {
            local_head: (head_short.to_owned(), head_commit_msg.1.to_owned()),
            remote_head: (upstream_short.to_owned(), upstream_commit_msg.1.to_owned()),
            tag: tag,
        });
    }

    info!("Branch {} has no upstream", head_short);

    Ok(RepoHeader {
        local_head: (head_short.to_owned(), head_commit_msg.1.to_owned()),
        remote_head: ("<no-upstream>".to_owned(), "-".to_owned()),
        tag: tag,
    })
}
