mod github;
mod top;

use github::GITHUB_COMMAND;
use serenity::framework::standard::macros::group;
use top::TOP_COMMAND;
#[group]
#[commands(github, top)]
pub(crate) struct SpecificChannel;
