use crate::git::RepoHeader;
use crate::widget::CheatSheetState;
use druid::{Data, Lens};

#[derive(Clone, Data, Lens, Debug)]
pub struct AppState {
    pub repo_header: RepoHeader,
    pub cheatsheet: CheatSheetState,
}
