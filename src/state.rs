use crate::git::RepoHeader;
use druid::{Data, Lens};

#[derive(Clone, Data, Lens)]
pub struct AppState {
    pub repo_header: RepoHeader,
}
