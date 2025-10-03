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
