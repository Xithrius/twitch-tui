use std::{collections::HashMap, sync::LazyLock};

use super::models::ReceivedTwitchEventBadges;

const VIP_BADGE: char = '\u{1F48E}';
const MODERATOR_BADGE: char = '\u{1F528}';
const SUBSCRIBER_BADGE: char = '\u{2B50}';
const PRIME_GAMING_BADGE: char = '\u{1F451}';

static BADGES: LazyLock<HashMap<&str, char>> = LazyLock::new(|| {
    HashMap::from_iter(vec![
        ("vip", VIP_BADGE),
        ("moderator", MODERATOR_BADGE),
        ("subscriber", SUBSCRIBER_BADGE),
        ("premium", PRIME_GAMING_BADGE),
    ])
});

pub fn retrieve_user_badges(badges: &Vec<ReceivedTwitchEventBadges>) -> String {
    let mut badges_str = String::new();

    for badge in badges {
        if let Some(badge_char) = BADGES.get(badge.set_id()) {
            badges_str.push(*badge_char);
        }
    }
    badges_str
}

// TODO: Tests
