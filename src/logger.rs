use std::time::{Duration, SystemTime};

use serenity::{client::Context, framework::standard::CommandResult, model::channel::Message, model::id::GuildId, model::id::UserId, prelude::Mentionable};
use sqlx::{query, Pool, Postgres, Transaction};

use crate::commands::{DbPool, get_time_as_unix_epoch};

pub(crate) async fn insert_message(
    message: &Message,
    guild_id: GuildId,
    mut con: Transaction<'_, Postgres>,
) -> CommandResult {
    let message_id = i64::from(message.id);
    let guild_id = i64::from(guild_id);
    let author_id = i64::from(message.author.id);
    let now = get_time_as_unix_epoch(SystemTime::now());
    if (!message.mention_roles.is_empty())
        || message.mentions.iter().any(|v| v.id != message.author.id)
    {
        query!(
            "INSERT INTO message_content (author_id,message_id, server_id, at_time, content) VALUES($1,$2,$3,$4,$5)",
            author_id,message_id,guild_id,now,message.content
        ).execute(&mut con).await?;
        for ping in message
            .mentions
            .iter()
            .filter(|v| v.id != message.author.id)
        {
            query!(
                "INSERT INTO pinged_users (user_id,message_id,server_id) VALUES ($1,$2,$3)",
                i64::from(ping.id),
                message_id,
                guild_id
            )
            .execute(&mut con)
            .await?;
        }
        for ping in &message.mention_roles {
            query!(
                "INSERT INTO pinged_roles (role_id,message_id,server_id) VALUES ($1,$2,$3)",
                (*ping.as_u64()) as i64,
                message_id,
                guild_id
            )
            .execute(&mut con)
            .await?;
        }
    }
    con.commit().await?;
    /*
    if message.
    */
    Ok(())
}

pub(crate) async fn cleanup_db(con: &Pool<Postgres>) -> sqlx::Result<()> {
    let time_stamp = get_time_as_unix_epoch(SystemTime::now() - Duration::from_secs(1800));
    query!(
        "DELETE FROM message_content
        CASCADE
        WHERE at_time < $1",
        time_stamp
    )
    .execute(con)
    .await
    .map(|_| ())
}

pub(crate) async fn check_deleted_message(
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
        WHERE message_id = $1
        AND server_id = $2
        AND (
            0 < (
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