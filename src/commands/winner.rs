use anyhow::Context as _;
use rand::seq::IndexedRandom;
use serenity::all::{CreateEmbed, Mention, ChannelId, UserId};

use crate::{Context, Error};
use super::utils::utils::get_channel_members;

const CONGRATULATIONS_GIF_URL: &str = "https://media.discordapp.net/attachments/1379075185935913001/1379093353865678958/congrats-leonardo-dicaprio.gif?ex=683fa505&is=683e5385&hm=9985763ded4578f7318e8b0dc6fe72c3b39085b385c4ebd5f8626e884cb176e4&=&width=688&height=290";
const GOLD_COLOR: u32 = 0xFFD700;

/// Get the voice channel ID of the command author
async fn get_author_voice_channel(ctx: &Context<'_>) -> Result<ChannelId, Error> {
    let author_id = ctx.author().id;
    Ok(ctx.guild()
        .and_then(|g| g.voice_states.get(&author_id)?.channel_id)
        .context("You must be in a voice channel to use this command")?)
}

/// Select a random winner from the channel members
async fn select_random_winner(
    ctx: Context<'_>,
    guild_id: serenity::model::id::GuildId,
    voice_channel_id: ChannelId
) -> Result<serenity::model::guild::Member, Error> {
    let channel_members = get_channel_members(guild_id, voice_channel_id, ctx).await?;

    if channel_members.is_empty() {
        return Err("There are no members in the voice channel!".into());
    }

    Ok(channel_members
        .choose(&mut rand::rng())
        .cloned()
        .context("Couldn't choose winner")?)
}

/// Create the winner announcement embed
fn create_winner_embed(
    winner_id: UserId,
    event: Option<String>,
    prize: Option<String>
) -> CreateEmbed {
    let winner_mention = Mention::User(winner_id).to_string();
    let mut embed = CreateEmbed::new()
        .title("🎉 Congratulations to our Winner! 🎉")
        .description(format!(
            "Everyone, please give a big round of applause to {} for winning our contest!",
            winner_mention
        ))
        .color(GOLD_COLOR)
        .image(CONGRATULATIONS_GIF_URL)
        .timestamp(chrono::Utc::now());

    // Add optional fields if provided
    if let Some(prize_text) = prize {
        embed = embed.field("Prize", prize_text, false);
    }
    if let Some(event_text) = event {
        embed = embed.field("Event", event_text, false);
    }

    embed
}

/// Pick winner from your voice channel
#[poise::command(prefix_command, slash_command)]
pub async fn winner(
    ctx: Context<'_>,
    #[description = "Event name"] event: Option<String>,
    #[description = "Prize for winner"] prize: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().context("Command must be used in a server")?;
    let voice_channel_id = get_author_voice_channel(&ctx).await?;

    // Select winner
    let winner = select_random_winner(ctx.clone(), guild_id, voice_channel_id).await?;

    // Create and send winner announcement
    let embed = create_winner_embed(winner.user.id, event, prize);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}
