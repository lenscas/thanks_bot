mod github;
mod top;
mod me;

use github::{GITHUB_COMMAND,BUG_COMMAND};
use serenity::framework::standard::macros::group;
use top::TOP_COMMAND;
use me::RANK_COMMAND;
#[group]
#[commands(github, top,bug,rank)]
pub(crate) struct SpecificChannel;
