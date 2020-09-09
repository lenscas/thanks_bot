mod commands;
mod logger;

use std::time::Duration;

use async_trait::async_trait;
use dotenv::var;
use logger::{cleanup_db, insert_message};
use serenity::{
    framework::standard::{macros::hook, CommandResult, StandardFramework},
    model::{channel::Message, gateway::Ready},
};
use serenity::{model::id::UserId, prelude::*};
use sqlx::{query, PgPool};

use commands::{moderator_only, MY_HELP};
use futures::stream::StreamExt;

use crate::commands::{BotId, DbPool, CONFIG_GROUP, GENERAL_GROUP};
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!(
            "{} is connected! with id {}",
            ready.user.name, ready.user.id
        );
        let mut data = ctx.data.write().await;
        data.insert::<BotId>(ready.user.id);
    }
    async fn message(&self, ctx: Context, new_message: Message) {
        let guild = match new_message.guild(&ctx).await {
            Some(x) => x,
            None => return,
        };
        if moderator_only(&ctx, &guild, &new_message.author)
            .await
            .unwrap_or(false)
        {
            return;
        }
        let transaction: Result<_, sqlx::Error> = (|| async {
            let data = ctx.data.read().await;
            let pool = data.get::<DbPool>().expect("No db pool?");
            Ok(pool.begin().await?)
        })()
        .await;
        let transaction = match transaction {
            Ok(x) => x,
            Err(x) => {
                let _ = dbg!(x);
                return;
            }
        };
        if let Err(x) = insert_message(&new_message, guild.id, transaction).await {
            let _ = dbg!(x);
        }
    }
    async fn message_delete(
        &self,
        ctx: Context,

        channel_id: serenity::model::id::ChannelId,
        message_id: serenity::model::id::MessageId,
    ) {
        let x = check_deleted_message(&ctx, channel_id, message_id).await;
        if let Err(x) = x {
            let _ = dbg!(x);
            let _ = dbg!(
                channel_id
                    .say(
                        ctx,
                        "Found a deleted message but something has gone wrong when processing"
                    )
                    .await
            );
        }
    }

    async fn message_update(
        &self,
        _ctx: Context,
        _old_if_available: Option<Message>,
        _new: Option<Message>,
        _event: serenity::model::event::MessageUpdateEvent,
    ) {
    }
}

async fn check_deleted_message(
    ctx: &Context,
    channel_id: serenity::model::id::ChannelId,
    message_id: serenity::model::id::MessageId,
) -> CommandResult {
    let guild = match channel_id.to_channel_cached(&ctx).await {
        Some(x) => match x.guild() {
            Some(x) => x,
            None => return Ok(()),
        },
        None => {
            return Ok(());
        }
    };
    dbg!("got here");
    let con = {
        let data = ctx.data.read().await;
        let pool = data.get::<DbPool>().expect("No db pool?");
        pool.acquire().await
    };

    let mut con = match con {
        Ok(x) => x,
        Err(x) => {
            channel_id
                .say(
                    ctx,
                    "Detected removed message, but couldn't check for ghost pings.",
                )
                .await?;
            return Err(x.into());
        }
    };

    dbg!("and got here");

    let message = query!(
        "
        SELECT 
            author_id,content 
        FROM message_content
        WHERE 0 < (
            SELECT count(*) AS role_count
            FROM pinged_roles 
            WHERE message_id = $1
            AND server_id = $2
        )
        OR 0 < (
            SELECT count(*) AS user_count
            FROM pinged_users 
            WHERE message_id = $1
            AND server_id = $2
        )
    ",
        i64::from(message_id),
        i64::from(guild.guild_id)
    )
    .fetch_optional(&mut con)
    .await?;
    dbg!((i64::from(message_id), i64::from(guild.guild_id)));
    let (author, content) = if let Some(x) = dbg!(message) {
        (UserId::from(x.author_id as u64).mention(), x.content)
    } else {
        return Ok(());
    };
    channel_id
        .send_message(ctx, |v| {
            v.embed(|x| {
                x.title("Ghost ping detected").description(format!(
                    "Author: {}
Content: {}
",
                    author, content
                ))
            })
        })
        .await?;
    Ok(())
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Could not read .env file");
    let discord_token = var("DISCORD_TOKEN").expect("DISCORD_TOKEN is not set.");
    let db_url = var("DATABASE_URL").expect("DATABASE_URL is not set.");
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Couldn't connect to database");
    let pool_db_cleanup = pool.clone();

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .prefix("!")
                // In this case, if "," would be first, a message would never
                // be delimited at ", ", forcing you to trim your arguments if you
                // want to avoid whitespaces at the start of each.
                .delimiters(vec![", ", ","])
        })
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&CONFIG_GROUP)
        .bucket("potentially_big_output", |b| {
            b.delay(10).time_span(120).limit(2)
        })
        .await
        .bucket("potentially_big_output_ever_channel", |b| {
            b.delay(30).time_span(120).limit(2)
        })
        .await
        .after(after);
    let mut client = Client::new(&discord_token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<DbPool>(pool);
    }

    let client_thread = async {
        if let Err(why) = client.start().await {
            println!("Client error: {:?}", why);
        }
    };

    let cleanup_thread = tokio::time::interval(Duration::from_secs(1800)).for_each(|_| async {
        println!("Start cleanup db");
        match cleanup_db(&pool_db_cleanup).await {
            Ok(_) => println!("cleaned up db"),
            Err(x) => {
                println!("Error during cleanup");
                let _ = dbg!(x);
            }
        }
    });

    futures::future::join(client_thread, cleanup_thread).await;
}
