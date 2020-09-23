mod config;
mod delete;

use config::SET_DELAY_COMMAND;
use delete::DELETE_COMMAND;
use serenity::framework::standard::macros::group;
#[group]
#[commands(set_delay, delete)]
pub(crate) struct Moderators;