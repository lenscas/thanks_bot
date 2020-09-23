mod github;
mod top;

use github::GITHUB_COMMAND;
use top::TOP_COMMAND;
use serenity::framework::standard::macros::group;
#[group]
#[commands(github,top)]
pub(crate) struct SpecificChannel;