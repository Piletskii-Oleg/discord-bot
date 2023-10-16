use crate::{CommandCounter, ShardManagerContainer};
use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::Message;
use serenity::prelude::Context;
use serenity::utils::{content_safe, Color, ContentSafeOptions};
use std::fmt::Write;

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
#[description("Sets a role with the color given in the format 0xRRGGBB with the name being caller's Discord nickname. If it already exists, changes its color.")]
#[usage("0xRRGGBB")]
#[example("0x34EB64")]
async fn set_role(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let color_code = args.current().unwrap_or_default();

    match i32::from_str_radix(color_code.trim_start_matches("0x"), 16) {
        Err(parse_error) => {
            println!("Couldn't parse color. Reason: {:?}", parse_error);
            msg.reply(ctx, "Provide a valid argument, in the format '0xRRGGBB'")
                .await?;
            Err(parse_error.into())
        }
        Ok(code) => {
            let color = Color::from(code);
            if let Err(why) = modify_role(ctx, msg, color).await {
                println!("Couldn't assign role. Reason: {:?}", why);
                msg.reply(ctx, "Couldn't assign role. Try again.").await?;
            }
            Ok(())
        }
    }
}

async fn modify_role(ctx: &Context, msg: &Message, color: Color) -> CommandResult {
    let mut member = msg.member(ctx).await?;
    let guild = msg.guild(ctx).unwrap();
    let user = &member.user;

    if let Some(role) = guild.role_by_name(&user.name) {
        let role = guild
            .edit_role(ctx, role.id, |role| role.colour(color.0 as u64))
            .await?;
        if member
            .roles(ctx)
            .is_some_and(|roles| !roles.contains(&role))
        {
            member.add_role(ctx, role.id).await?;
        }
    } else {
        let role = guild
            .create_role(ctx, |role| {
                role.colour(color.0 as u64).name(&user.name).hoist(false)
            })
            .await?;
        member.add_role(ctx, role).await?;
    }

    msg.reply(
        ctx,
        format!(
            "Successfully set role with the color 0x{} to user {}",
            color.hex(),
            &member.user.name
        ),
    )
    .await?;
    Ok(())
}
