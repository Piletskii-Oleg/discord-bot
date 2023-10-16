use chrono::{DateTime, Datelike, FixedOffset, ParseResult};
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use sqlx::Connection;

#[group]
#[commands(birthday)]
struct Birthday;

#[command]
#[sub_commands(add, edit, remove, mod_menu)]
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

fn parse_date(date: &str) -> ParseResult<DateTime<FixedOffset>> {
    let mut date = date.to_owned();
    date.push_str(".1970 00:00:00 +0300");
    DateTime::parse_from_str(&date, "%d.%m.%Y %H:%M:%S %z")
}

#[command]
async fn add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if let Some(maybe_birthday) = args.current() {
        match parse_date(maybe_birthday) {
            Ok(birthday) => {
                let mut db = sqlx::SqliteConnection::connect("database.sqlite").await?;

                let (day, month) = (birthday.day(), birthday.month());
                let author_copy = msg.author.name.to_string();
                let user_id = msg.author.id.0.to_string();

                let query = sqlx::query!(
                    "INSERT INTO birthdays (user_id, birth_day, birth_month, name) VALUES (?, ?, ?, ?)",
                    user_id,
                    day,
                    month,
                    author_copy
                )
                .execute(&mut db)
                .await;

                match query {
                    Err(query_error) => {
                        println!("Couldn't add birthday. Reason: {:?}", query_error);
                        msg.reply(ctx, "Couldn't add birthday. Probably it already exists.")
                            .await?;
                    }
                    Ok(_) => {
                        let success =
                            format!("Added {}'s birthday {}.{}", msg.author.name, day, month);
                        println!("{success}");
                        msg.channel_id.say(ctx, success).await?;
                    }
                }
            }
            Err(parse_error) => {
                println!("Invalid argument: {}", parse_error);
                msg.channel_id
                    .say(ctx, "Provide a valid date in the format dd.mm.")
                    .await?;
            }
        }
    } else {
        println!("No birth day-month argument was provided.");
        msg.channel_id
            .say(ctx, "Provide a valid date in the format dd.mm.!!!")
            .await?;
    }

    Ok(())
}

#[command]
async fn edit(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.reply(ctx, "edited").await?;
    Ok(())
}

#[command]
async fn remove(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    Ok(())
}
