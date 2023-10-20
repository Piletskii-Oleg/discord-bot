use chrono::{DateTime, Datelike, FixedOffset, ParseResult};
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use sqlx::{Connection, SqliteConnection};

#[group]
#[commands(birthday)]
struct Birthday;

#[command]
#[sub_commands(add, edit, remove, mod_menu, get)]
async fn birthday(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.reply(ctx, "what???").await?;
    Ok(())
}

#[command]
#[required_permissions("MANAGE_ROLES")]
async fn mod_menu(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.reply(ctx, "mod_menu").await?;
    Ok(())
}

#[command]
#[max_args(1)]
#[delimiters(" ", ",", ", ")]
async fn add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut db = sqlx::SqliteConnection::connect("database.sqlite").await?;
    let user_id = msg.author.id.0.to_string();
    if is_user_in(&mut db, &user_id).await {
        msg.channel_id
            .say(ctx, "User's birthday already exists.")
            .await?;
        return Ok(());
    }

    if let Some(maybe_birthday) = args.current() {
        match parse_date(maybe_birthday) {
            Ok(birthday) => {
                let (day, month) = (birthday.day(), birthday.month());
                let author_copy = msg.author.name.to_string();

                let query = sqlx::query!(
                    "INSERT INTO birthdays (user_id, birth_day, birth_month, name) VALUES (?, ?, ?, ?)",
                    user_id,
                    day,
                    month,
                    author_copy
                )
                    .execute(&mut db)
                    .await;


                if let Err(query_error) = query {
                    msg.reply(ctx, "Couldn't add birthday. Probably it already exists.")
                        .await?;
                    return Err(query_error.into());
                }

                let success = format!(
                    "Added birthday for user {}: {}",
                    msg.author.name,
                    birthday.format("%B %d")
                );
                println!("{success}");
                msg.channel_id.say(ctx, success).await?;
            }
            Err(parse_error) => {
                msg.channel_id
                    .say(ctx, "Provide a valid date in the format dd.mm.")
                    .await?;
                return Err(parse_error.into());
            }
        }
    } else {
        println!("No birth day-month argument was provided.");
        msg.channel_id
            .say(ctx, "Provide a valid date in the format dd.mm.")
            .await?;
    }

    Ok(())
}

#[command]
async fn edit(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut db = sqlx::SqliteConnection::connect("database.sqlite").await?;
    let user_id = msg.author.id.0.to_string();
    if !is_user_in(&mut db, &user_id).await {
        msg.channel_id
            .say(
                ctx,
                "Birthday not found. Please add birthday using ~birthday add DD.MM",
            )
            .await?;
        return Ok(());
    }

    if let Some(maybe_birthday) = args.current() {
        match parse_date(maybe_birthday) {
            Ok(birthday) => {
                let (day, month) = (birthday.day(), birthday.month());
                let query = sqlx::query!(
                    "UPDATE birthdays SET birth_day = ?, birth_month = ? WHERE user_id = ?",
                    day,
                    month,
                    user_id
                )
                .execute(&mut db)
                .await?;

                let success = format!(
                    "Edited birthday for user {}: {}",
                    msg.author.name,
                    birthday.format("%B %d")
                );
                println!("{success}");
                msg.channel_id.say(ctx, success).await?;
            }
            Err(parse_error) => {
                msg.channel_id
                    .say(ctx, "Provide a valid date in the format dd.mm.")
                    .await?;
                return Err(parse_error.into());
            }
        }
    }
    Ok(())
}

#[command]
async fn remove(ctx: &Context, msg: &Message) -> CommandResult {
    let mut db = sqlx::SqliteConnection::connect("database.sqlite").await?;
    let user_id = msg.author.id.0.to_string();
    if !is_user_in(&mut db, &user_id).await {
        msg.channel_id
            .say(
                ctx,
                "Birthday not found. Please add birthday using ~birthday add DD.MM",
            )
            .await?;
        return Ok(());
    }

    let remove_query = sqlx::query!("DELETE FROM birthdays WHERE user_id = ?", user_id).execute(&mut db).await;
    if let Err(removal) = remove_query {
        println!("Couldn't remove {}'s birthday.", msg.author.name);
        msg.channel_id.say(ctx, "Couldn't remove your birthday. Try again.").await?;
        return Err(removal.into());
    }

    msg.channel_id.say(ctx, "Successfully removed your birthday.").await?;
    Ok(())
}

#[command]
async fn get(ctx: &Context, msg: &Message) -> CommandResult {
    let mut db = sqlx::SqliteConnection::connect("database.sqlite").await?;
    let user_id = msg.author.id.0.to_string();
    if is_user_in(&mut db, &user_id).await {
        let (day, month) = get_birthday(&mut db, &user_id).await?;
        let date = format!("{}.{}", day, month);
        let date = parse_date(&date)?;
        msg.channel_id.say(ctx, format!("Your birthday is set to be {}", date.format("%B %d"))).await?;
        Ok(())
    } else {
        msg.channel_id.say(ctx, "No birthday found.").await?;
        Err(format!("No birthday found for user {}", &msg.author.name).into())
    }
}

async fn is_user_in(db: &mut SqliteConnection, user_id: &str) -> bool {
    match sqlx::query!("SELECT * FROM birthdays WHERE user_id = ?", user_id)
        .fetch_one(db)
        .await
    {
        Ok(_) => true,
        Err(_) => false,
    }
}

async fn get_birthday(db: &mut SqliteConnection, user_id: &str) -> Result<(u64, u64), sqlx::Error> {
    sqlx::query!(
        "SELECT birth_day, birth_month FROM birthdays WHERE user_id = ?",
        user_id
    )
    .fetch_one(db)Ok(())
    .await
    .and_then(|x| Ok((x.birth_day as u64, x.birth_month as u64)))
}

fn parse_date(date: &str) -> ParseResult<DateTime<FixedOffset>> {
    let mut date = date.to_owned();
    date.push_str(".1970 00:00:00 +0300");
    DateTime::parse_from_str(&date, "%d.%m.%Y %H:%M:%S %z")
}
