use crate::state::{AppState, Command};
use crate::theme;
use anyhow::{anyhow, Context, Result};
use druid::widget::{Align, Container, CrossAxisAlignment, Flex, Label, SizedBox};
use druid::{Data, Env, Widget, WidgetExt};
use git2::{
    BranchType, DescribeFormatOptions, DescribeOptions, Reference, Repository, Status,
    StatusOptions, Statuses,
};
use im::{vector, Vector};
use log::{debug, info};

pub fn get_commit_from_ref(repo: &Repository, reference: &Reference) -> Result<(String, String)> {
    let oid = reference.target().ok_or(anyhow!("No oid on ref"))?;
    let commit = repo.find_commit(oid)?;
    let msg = commit
        .message()
        .ok_or(anyhow!("No commit message"))?
        .split("\n")
        .next()
        .unwrap();

    debug!("Ref commit msg: {}", msg);

    Ok((format!("{}", oid), msg.to_owned()))
}

pub fn get_latest_tag(repo: &Repository) -> String {
    let mut opts = DescribeOptions::new();
    let opts = opts.describe_tags();
    let mut format_opts = DescribeFormatOptions::new();
    let format_opts = format_opts.abbreviated_size(0);
    if let Ok(describe) = repo.describe(opts) {
        return describe.format(Some(format_opts)).unwrap_or("".to_owned());
    }

    "<no-tags>".to_owned()
}

pub fn get_branches(repo: &Repository) -> (Vector<String>, Vector<String>) {
    let branches = repo.branches(None);
    if branches.is_err() {
        info!("No branches in repo");
        return (vector![], vector![]);
    }

    let (mut local, mut remote) = (vector![], vector![]);

    let branches = branches.unwrap();
    for b in branches {
        if b.is_ok() {
            let (branch, typ) = b.unwrap();
            if let Ok(Some(branch_name)) = branch.name() {
                match typ {
                    BranchType::Local => local.push_back(branch_name.to_owned()),
                    BranchType::Remote => remote.push_back(branch_name.to_owned()),
                }
            }
        }
    }

    (local, remote)
}

pub fn get_statuses(repo: &Repository) -> Result<Statuses> {
    let mut status_opts = StatusOptions::new();
    let mut status_opts = status_opts.include_untracked(true);
    repo.statuses(Some(&mut status_opts))
        .context("Failed to get status")
}

/// Handle commands from the ui. Repository state will change depending
/// on the issued command
pub fn execute_cmd(repo: &Repository, cmd: Command, selection: &str) {
    match cmd {
        Command::BranchCheckout => {
            checkout_branch(repo, selection);
        }
        _ => (),
    }
}

fn checkout_branch(repo: &Repository, name: &str) {
    debug!("Checking out {}", name);
    let head = "refs/heads/".to_owned() + name;
    repo.set_head(&head);
    repo.checkout_head(None);
}
