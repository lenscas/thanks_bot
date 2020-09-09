use std::time::{SystemTime, Duration};

use serenity::{client::Context, framework::standard::CommandResult, model::channel::Message, model::id::GuildId};
use sqlx::{ query, Transaction, Postgres, Pool};

use crate::commands::get_time_as_unix_epoch;

pub(crate) async fn insert_message(message: &Message, guild_id : GuildId,mut con: Transaction<'_, Postgres>) -> CommandResult
{
    
    let message_id =i64::from(message.id);
    let guild_id = i64::from(guild_id);
    let author_id = i64::from(message.author.id);
    let now = get_time_as_unix_epoch(SystemTime::now());
    if (!message.mention_roles.is_empty()) || message.mentions.iter().any(|v| v.id != message.author.id) {
        query!(
            "INSERT INTO message_content (author_id,message_id, server_id, at_time, content) VALUES($1,$2,$3,$4,$5)",
            author_id,message_id,guild_id,now,message.content
        ).execute(&mut con).await?;
        for ping in message.mentions.iter().filter(|v| v.id != message.author.id) {
            query!(
                "INSERT INTO pinged_users (user_id,message_id,server_id) VALUES ($1,$2,$3)",
                i64::from(ping.id),message_id,guild_id
            ).execute(&mut con).await?;
        }
        for ping in &message.mention_roles {
            query!(
                "INSERT INTO pinged_roles (role_id,message_id,server_id) VALUES ($1,$2,$3)",
                (*ping.as_u64()) as i64,message_id,guild_id
            ).execute(&mut con).await?;
        }
    }
    con.commit().await?;
    /*
    if message.
    */
    Ok(())
}

pub(crate) async fn cleanup_db(con : &Pool<Postgres>) -> sqlx::Result<()> {
    let time_stamp= get_time_as_unix_epoch(SystemTime::now() - Duration::from_secs(1800));
    query!(
        "DELETE FROM message_content
        CASCADE
        WHERE at_time < $1",time_stamp
    ).execute(con).await.map(|_|())
}