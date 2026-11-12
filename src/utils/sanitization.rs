use crate::ui::statics::WEBSOCKET_SERVER_PREFIX;

/// If the channel name contains the broadcaster login and stream title, return broadcaster login
///
/// TODO: This is a temporary function that "fixes" an underlying issue of follower channel selection
pub fn clean_channel_name(channel: &str) -> String {
    channel
        .split_once(':')
        .map_or(channel, |(a, _)| a)
        .trim()
        .to_owned()
}

/// Checks if the input server string is a valid string for connecting to a websocket server
pub fn is_valid_websocket_server(server: &str) -> bool {
    server.starts_with(WEBSOCKET_SERVER_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_channel_already_clean() {
        let channel = "xithrius";
        let cleaned_channel = clean_channel_name(channel);
        assert_eq!(cleaned_channel, channel);
    }

    #[test]
    fn test_clean_channel_non_clean_channel() {
        let channel = "xithrius    : stream name";
        let cleaned_channel = clean_channel_name(channel);
        assert_eq!(cleaned_channel, "xithrius");
    }

    #[test]
    fn test_valid_websocket_server() {
        let server = "wss://eventsub.wss.twitch.tv/ws";
        assert!(is_valid_websocket_server(server));
    }

    #[test]
    fn test_invalid_websocket_server() {
        let server = "irc.chat.twitch.tv";
        assert!(!is_valid_websocket_server(server));
    }
}
