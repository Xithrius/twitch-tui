use std::str::FromStr;

use color_eyre::{Result, eyre::ContextCompat};
use tokio::sync::mpsc::Sender;
use tracing::debug;

use super::super::oauth::TwitchOauth;
use crate::{
    events::Event,
    handlers::data::DataBuilder,
    twitch::{
        api::{
            BroadcasterQuery, ModeratorQuery,
            channel_information::{
                UpdateChannelInformationPayload, get_game_id, update_channel_information,
            },
            channels::get_channel_id,
            chat_settings::{UpdateTwitchChatSettingsPayload, update_chat_settings},
            clear::{DeleteMessageQuery, delete_twitch_messages},
            commercial::{CommercialPayload, start_commercial},
            mods::{mod_twitch_user, unmod_twitch_user},
            raids::{RaidQuery, raid_twitch_user, unraid_twitch_user},
            shoutouts::{ShoutoutQuery, shoutout_twitch_user},
            timeouts::{TimeoutPayload, UnbanQuery, timeout_twitch_user, unban_twitch_user},
            vips::{unvip_twitch_user, vip_twitch_user},
        },
        context::TwitchWebsocketContext,
        handlers::twitch_commands::TwitchCommand,
    },
};

pub async fn handle_command_message(
    context: &TwitchWebsocketContext,
    event_tx: &Sender<Event>,
    user_command: &str,
) -> Result<()> {
    let Ok(command) = TwitchCommand::from_str(user_command) else {
        event_tx
            .send(
                DataBuilder::system(format!(
                    "Command /{user_command} either does not exist, or is not supported"
                ))
                .into(),
            )
            .await?;

        return Ok(());
    };

    let twitch_client = context
        .twitch_client()
        .context("Twitch client could not be found when sending command")?;

    let channel_id = context
        .channel_id()
        .context("Channel ID could not be found when sending command")?;

    let user_id = context
        .oauth()
        .and_then(TwitchOauth::user_id)
        .context("Twitch OAuth could not be found when sending command")?;

    let command_message = match command {
        TwitchCommand::Clear => {
            let delete_message_query = DeleteMessageQuery::new(channel_id.clone(), user_id, None);
            delete_twitch_messages(&twitch_client, delete_message_query).await?;

            "Chat was cleared for non-Moderators viewing this room".to_string()
        }
        TwitchCommand::Ban(username, reason) => {
            let target_user_id = get_channel_id(&twitch_client, &username).await?;

            let ban_query = ModeratorQuery::new(channel_id.clone(), user_id);

            let ban_payload = TimeoutPayload::new(target_user_id, None, reason.clone());

            timeout_twitch_user(&twitch_client, ban_query, ban_payload).await?;

            reason.map_or_else(
                || format!("User {username} banned."),
                |reason| format!("User {username} banned. Reason: {reason}"),
            )
        }
        TwitchCommand::Timeout(username, duration, reason) => {
            let target_user_id = get_channel_id(&twitch_client, &username).await?;

            let timeout_query = ModeratorQuery::new(channel_id.clone(), user_id);

            let timeout_payload =
                TimeoutPayload::new(target_user_id, Some(duration), reason.clone());

            timeout_twitch_user(&twitch_client, timeout_query, timeout_payload).await?;

            reason.map_or_else(
                || format!("User {username} timed out for {duration} seconds."),
                |reason| {
                    format!("User {username} timed out for {duration} seconds. Reason: {reason}")
                },
            )
        }
        TwitchCommand::Unban(username) => {
            let target_user_id = get_channel_id(&twitch_client, &username).await?;

            let unban_query = UnbanQuery::new(channel_id.clone(), user_id, target_user_id);

            unban_twitch_user(&twitch_client, unban_query).await?;

            format!("User {username} unbanned")
        }
        TwitchCommand::Raid(username) => {
            let target_user_id = get_channel_id(&twitch_client, &username).await?;

            let raid_query = RaidQuery::new(channel_id.clone(), target_user_id);

            raid_twitch_user(&twitch_client, raid_query).await?;

            format!("Raid to {username} created")
        }
        TwitchCommand::Unraid => {
            unraid_twitch_user(&twitch_client, channel_id.clone()).await?;

            "Raid cancelled".to_string()
        }
        TwitchCommand::Followers(duration) => {
            let update_query = ModeratorQuery::new(channel_id.clone(), user_id);

            let update_payload = UpdateTwitchChatSettingsPayload::new_follower_mode(true, duration);

            update_chat_settings(&twitch_client, update_query, update_payload).await?;

            duration.map_or_else(
                || "Enabled followers-only mode for this room".to_string(),
                |duration| format!("Enabled {duration} minutes followers-only mode for this room"),
            )
        }
        TwitchCommand::FollowersOff => {
            let update_query = ModeratorQuery::new(channel_id.clone(), user_id);

            let update_payload = UpdateTwitchChatSettingsPayload::new_follower_mode(false, None);

            update_chat_settings(&twitch_client, update_query, update_payload).await?;

            "Disabled followers-only mode for this room".to_string()
        }
        TwitchCommand::Slow(duration) => {
            let update_query = ModeratorQuery::new(channel_id.clone(), user_id);

            let update_payload =
                UpdateTwitchChatSettingsPayload::new_slow_mode(true, Some(duration));

            update_chat_settings(&twitch_client, update_query, update_payload).await?;

            format!("Enabled {duration}-second slow mode for this room")
        }
        TwitchCommand::SlowOff => {
            let update_query = ModeratorQuery::new(channel_id.clone(), user_id);

            let update_payload = UpdateTwitchChatSettingsPayload::new_slow_mode(false, None);

            update_chat_settings(&twitch_client, update_query, update_payload).await?;

            "Disabled slow mode for this room".to_string()
        }
        TwitchCommand::Subscribers => {
            let update_query = ModeratorQuery::new(channel_id.clone(), user_id);

            let update_payload = UpdateTwitchChatSettingsPayload::new_subscriber_mode(true);

            update_chat_settings(&twitch_client, update_query, update_payload).await?;

            "Enabled subscribers-only mode for this room".to_string()
        }
        TwitchCommand::SubscribersOff => {
            let update_query = ModeratorQuery::new(channel_id.clone(), user_id);

            let update_payload = UpdateTwitchChatSettingsPayload::new_subscriber_mode(false);

            update_chat_settings(&twitch_client, update_query, update_payload).await?;

            "Disabled subscribers-only mode for this room".to_string()
        }
        TwitchCommand::EmoteOnly => {
            let update_query = ModeratorQuery::new(channel_id.clone(), user_id);

            let update_payload = UpdateTwitchChatSettingsPayload::new_emote_only_mode(true);

            update_chat_settings(&twitch_client, update_query, update_payload).await?;

            "Enabled emote-only mode for this room".to_string()
        }
        TwitchCommand::EmoteOnlyOff => {
            let update_query = ModeratorQuery::new(channel_id.clone(), user_id);

            let update_payload = UpdateTwitchChatSettingsPayload::new_emote_only_mode(false);

            update_chat_settings(&twitch_client, update_query, update_payload).await?;

            "Disabled emote-only mode for this room".to_string()
        }
        TwitchCommand::UniqueChat => {
            let update_query = ModeratorQuery::new(channel_id.clone(), user_id);

            let update_payload = UpdateTwitchChatSettingsPayload::new_unique_chat_mode(true);

            update_chat_settings(&twitch_client, update_query, update_payload).await?;

            "Enabled unique-chat mode for this room".to_string()
        }
        TwitchCommand::UniqueChatOff => {
            let update_query = ModeratorQuery::new(channel_id.clone(), user_id);

            let update_payload = UpdateTwitchChatSettingsPayload::new_unique_chat_mode(false);

            update_chat_settings(&twitch_client, update_query, update_payload).await?;

            "Disabled unique-chat mode for this room".to_string()
        }
        TwitchCommand::Vip(username) => {
            let target_user_id = get_channel_id(&twitch_client, &username).await?;

            let vip_query = BroadcasterQuery::new(channel_id.clone(), target_user_id);

            vip_twitch_user(&twitch_client, vip_query).await?;

            format!("Added {username} as a VIP of the channel")
        }
        TwitchCommand::Unvip(username) => {
            let target_user_id = get_channel_id(&twitch_client, &username).await?;

            let unvip_query = BroadcasterQuery::new(channel_id.clone(), target_user_id);

            unvip_twitch_user(&twitch_client, unvip_query).await?;

            format!("Removed {username} as a VIP of the channel")
        }
        TwitchCommand::Mod(username) => {
            let target_user_id = get_channel_id(&twitch_client, &username).await?;

            let mod_query = BroadcasterQuery::new(channel_id.clone(), target_user_id);

            mod_twitch_user(&twitch_client, mod_query).await?;

            format!("Granted moderator privledges to {username}")
        }
        TwitchCommand::Unmod(username) => {
            let target_user_id = get_channel_id(&twitch_client, &username).await?;

            let unmod_query = BroadcasterQuery::new(channel_id.clone(), target_user_id);

            unmod_twitch_user(&twitch_client, unmod_query).await?;

            format!("Removed {username} as a moderator of this channel")
        }
        TwitchCommand::Shoutout(username) => {
            let target_user_id = get_channel_id(&twitch_client, &username).await?;

            let shoutout_query = ShoutoutQuery::new(channel_id.clone(), target_user_id, user_id);

            shoutout_twitch_user(&twitch_client, shoutout_query).await?;

            format!("Gave a shoutout to {username}")
        }
        TwitchCommand::Commercial(duration) => {
            let commercial_payload = CommercialPayload::new(channel_id.clone(), duration);
            start_commercial(&twitch_client, commercial_payload).await?;

            format!("Started a commercial for {duration} seconds")
        }
        TwitchCommand::Title(title) => {
            let update_payload = UpdateChannelInformationPayload::new_title(&title);

            update_channel_information(&twitch_client, channel_id.clone(), update_payload).await?;

            format!("The title of the stream was changed to {title}")
        }
        TwitchCommand::Category(game_name) => {
            let game_id = get_game_id(&twitch_client, &game_name).await?;

            let update_payload = UpdateChannelInformationPayload::new_category(&game_id);

            update_channel_information(&twitch_client, channel_id.clone(), update_payload).await?;

            format!("The category of the stream was changed to {game_name}")
        }
    };

    debug!("Sending command message: {command_message}");
    event_tx
        .send(DataBuilder::twitch(command_message).into())
        .await?;

    Ok(())
}
