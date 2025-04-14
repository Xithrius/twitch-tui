const VIP_BADGE: char = '\u{1F48E}';
const MODERATOR_BADGE: char = '\u{1F528}';
const SUBSCRIBER_BADGE: char = '\u{2B50}';
const PRIME_GAMING_BADGE: char = '\u{1F451}';

// pub fn retrieve_user_badges(name: &mut String, message: &Message, badges_enabled: bool) {
//     let mut badges = String::new();

//     if let Some(ref tags) = message.tags {
//         let mut vip_badge = None;
//         let mut moderator_badge = None;
//         let mut subscriber_badge = None;
//         let mut prime_badge = None;
//         let mut display_name = None;

//         for tag in tags {
//             if tag.0 == *"display-name" {
//                 if let Some(ref value) = tag.1 {
//                     display_name = Some(value.to_string());

//                     // Assuming when found, that there is only one instance of
//                     // the 'display-name' tag.
//                     if !badges_enabled {
//                         break;
//                     }
//                 }
//             }

//             if tag.0 == *"badges" {
//                 if let Some(ref value) = tag.1 {
//                     if !value.is_empty() && value.contains("vip") {
//                         vip_badge = Some(VIP_BADGE);
//                     }
//                     if !value.is_empty() && value.contains("moderator") {
//                         moderator_badge = Some(MODERATOR_BADGE);
//                     }
//                     if !value.is_empty() && value.contains("subscriber") {
//                         subscriber_badge = Some(SUBSCRIBER_BADGE);
//                     }
//                     if !value.is_empty() && value.contains("premium") {
//                         prime_badge = Some(PRIME_GAMING_BADGE);
//                     }
//                 }
//             }
//         }

//         if let Some(display_name) = display_name {
//             *name = display_name;

//             if !badges_enabled {
//                 return;
//             }
//         }

//         if let Some(badge) = vip_badge {
//             badges.push(badge);
//         }

//         if let Some(badge) = moderator_badge {
//             badges.push(badge);
//         }

//         if let Some(badge) = subscriber_badge {
//             badges.push(badge);
//         }

//         if let Some(badge) = prime_badge {
//             badges.push(badge);
//         }

//         if !badges.is_empty() {
//             *name = badges.clone() + name;
//         }
//     }
// }
