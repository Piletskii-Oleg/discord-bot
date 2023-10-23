use crate::{CommandCounter, ShardManagerContainer};
use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandError, CommandResult};
use serenity::model::prelude::Message;
use serenity::prelude::Context;
use serenity::utils::{content_safe, Color, ContentSafeOptions};
use std::fmt::Write;

const PATTERNS: [&str; 2] = ["0x", "#"];

#[group]
#[commands(say, commands, latency, set_role)]
struct General;

// Commands can be created via the attribute `#[command]` macro.
#[command]
// Options are passed via subsequent attributes.
// Make this command use the "complicated" bucket.
#[bucket = "complicated"]
async fn commands(ctx: &Context, msg: &Message) -> CommandResult {
    let mut contents = "Commands used:\n".to_string();

    let data = ctx.data.read().await;
    let counter = data
        .get::<CommandCounter>()
        .expect("Expected CommandCounter in TypeMap.");

    for (k, v) in counter {
        writeln!(contents, "- {name}: {amount}", name = k, amount = v)?;
    }

    msg.channel_id.say(&ctx.http, &contents).await?;

    Ok(())
}

#[command]
async fn latency(ctx: &Context, msg: &Message) -> CommandResult {
    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager")
                .await?;

            return Ok(());
        }
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    // Shards are backed by a "shard runner" responsible for processing events
    // over the shard, so we'll get the information about the shard runner for
    // the shard this command was sent over.
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        }
    };

    msg.reply(ctx, &format!("The shard latency is {:?}", runner.latency))
        .await?;

    Ok(())
}

#[command]
async fn say(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.single_quoted::<String>() {
        Ok(x) => {
            let settings = if let Some(guild_id) = msg.guild_id {
                ContentSafeOptions::default()
                    .clean_channel(false)
                    .display_as_member_from(guild_id)
            } else {
                ContentSafeOptions::default()
                    .clean_channel(false)
                    .clean_role(false)
            };

            let content = content_safe(&ctx.cache, x, &settings, &msg.mentions);

            msg.channel_id.say(&ctx.http, &content).await?;
        }
        Err(_) => {
            msg.reply(ctx, "An argument is required to run this command.")
                .await?;
        }
    };

    Ok(())
}

#[command]
#[description("Sets a role with the color given in the hex format with the name being caller's Discord nickname. If it already exists, changes its color.")]
#[usage("#RRGGBB")]
#[example("#34EB61")]
#[max_args(1)]
async fn set_role(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let color_code = args.current().unwrap_or_default();

    let patterns_found = find_patterns(color_code);

    if let Some(&pattern) = patterns_found.first() {
        match i32::from_str_radix(color_code.trim_start_matches(pattern), 16) {
            Err(parse_error) => {
                println!("Couldn't parse color. Reason: {:?}", parse_error);
                msg.reply(
                    ctx,
                    "Provide a valid argument, in the hex format (#RRGGBB or 0xRRGGBB)",
                )
                .await?;
                Err(parse_error.into())
            }
            Ok(code) => {
                let color = Color::from(code);
                if let Err(why) = modify_role(ctx, msg, color).await {
                    msg.reply(ctx, "Couldn't assign role. Try again.").await?;
                    Err(why)
                } else {
                    Ok(())
                }
            }
        }
    } else {
        Err(CommandError::from("No arguments parsed"))
    }
}

fn find_patterns(color_code: &str) -> Vec<&&str> {
    PATTERNS
        .iter()
        .filter(|&&pattern| color_code.starts_with(pattern))
        .collect::<Vec<&&str>>()
}

async fn modify_role(ctx: &Context, msg: &Message, color: Color) -> CommandResult {
    let mut member = msg.member(ctx).await?;
    let guild = msg
        .guild(ctx)
        .ok_or(CommandError::from("Couldn't find guild in cache"))?;

    match guild.role_by_name(&member.user.name) {
        Some(role) => {
            let role = guild
                .edit_role(ctx, role.id, |role| role.colour(color.0 as u64))
                .await?;
            if member
                .roles(ctx)
                .is_some_and(|roles| !roles.contains(&role))
            {
                member.add_role(ctx, role.id).await?;
            }
        }
        None => {
            let role = guild
                .create_role(ctx, |role| {
                    role.colour(color.0 as u64)
                        .name(&member.user.name)
                        .hoist(false)
                })
                .await?;
            member.add_role(ctx, role).await?;
        }
    }

    msg.reply(
        ctx,
        format!(
            "Successfully set role with the color #{} to user {}",
            color.hex(),
            &member.user.name
        ),
    )
    .await?;
    Ok(())
}
