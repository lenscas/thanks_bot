use super::is_in_incorrect_channel;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

#[command]
#[aliases("paste", "code")]
#[description("Teaches users how to paste code neatly")]
#[usage("")]
#[example = ""]
#[help_available]
#[bucket = "potentially_big_output_ever_channel"]
pub(crate) async fn codeblock(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(
            &ctx.http,
            "Looks like something <https://www.google.com/> would know!",
        )
        .await?;
    Ok(())
}
