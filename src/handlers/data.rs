use std::{borrow::Cow, string::ToString};

use chrono::{offset::Local, DateTime};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use log::warn;
use once_cell::sync::Lazy;
use tui::{
    style::{Color, Color::Rgb, Modifier, Style},
    text::{Line, Span},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    emotes::{load_emote, Emotes, LoadedEmote},
    handlers::config::{FrontendConfig, Palette, Theme},
    ui::statics::NAME_MAX_CHARACTERS,
    utils::{
        colors::hsl_to_rgb,
        styles::{
            DATETIME_DARK, DATETIME_LIGHT, HIGHLIGHT_NAME_DARK, HIGHLIGHT_NAME_LIGHT, SYSTEM_CHAT,
        },
        text::split_cow_in_place,
    },
};

static FUZZY_FINDER: Lazy<SkimMatcherV2> = Lazy::new(SkimMatcherV2::default);

#[derive(Debug, Clone)]
pub struct EmoteData {
    pub name: String,
    pub index_in_message: usize,
    pub id: u32,
    pub pid: u32,
    pub layer: u16,
}

impl EmoteData {
    pub const fn new(emote: &LoadedEmote, name: String, idx: usize, layer: u16) -> Self {
        Self {
            name,
            index_in_message: idx,
            id: emote.hash,
            pid: emote.n,
            layer,
        }
    }
}

#[allow(dead_code)]
pub enum TwitchToTerminalAction {
    Message(MessageData),
    ClearChat,
    DeleteMessage(String),
}

#[derive(Debug, Clone)]
pub struct MessageData {
    pub time_sent: DateTime<Local>,
    pub author: String,
    pub system: bool,
    pub payload: String,
    pub emotes: Vec<EmoteData>,
    pub message_id: Option<String>,
}

impl MessageData {
    pub fn new(author: String, system: bool, payload: String, message_id: Option<String>) -> Self {
        Self {
            time_sent: Local::now(),
            author,
            system,
            payload,
            emotes: vec![],
            message_id,
        }
    }

    fn hash_username(&self, palette: &Palette) -> Color {
        let hash = f64::from(
            self.author
                .as_bytes()
                .iter()
                .map(|&b| u32::from(b))
                .sum::<u32>(),
        );

        let (hue, saturation, lightness) = match palette {
            Palette::Pastel => (hash % 360. + 1., 0.5, 0.75),
            Palette::Vibrant => (hash % 360. + 1., 1., 0.6),
            Palette::Warm => ((hash % 100. + 1.) * 1.2, 0.8, 0.7),
            Palette::Cool => ((hash % 100. + 1.).mul_add(1.2, 180.), 0.6, 0.7),
        };

        let rgb = hsl_to_rgb(hue, saturation, lightness);

        Rgb(rgb[0], rgb[1], rgb[2])
    }

    pub fn to_vec(
        &self,
        frontend_config: &FrontendConfig,
        width: usize,
        search_highlight: Option<&str>,
        username_highlight: Option<&str>,
    ) -> Vec<Line> {
        #[inline]
        fn highlight<'s>(
            line: Cow<'s, str>,
            start_index: &mut usize,
            search_highlight: &[usize],
            search_theme: Style,
            username_highlight: &[usize],
            username_theme: Style,
        ) -> Vec<Span<'s>> {
            // Fast path
            if search_highlight.is_empty() && username_highlight.is_empty() {
                return vec![Span::raw(line)];
            }

            // Slow path
            let lines = line
                .chars()
                .zip(*start_index..)
                .map(|(c, i)| {
                    if search_highlight
                        .binary_search(&(i.saturating_sub(1)))
                        .is_ok()
                    {
                        Span::styled(c.to_string(), search_theme)
                    } else if username_highlight.binary_search(&i).is_ok() {
                        Span::styled(c.to_string(), username_theme)
                    } else {
                        Span::raw(c.to_string())
                    }
                })
                .collect();
            *start_index += line.len();
            lines
        }

        // Theme styles
        let username_theme = match frontend_config.theme {
            Theme::Dark => HIGHLIGHT_NAME_DARK,
            _ => HIGHLIGHT_NAME_LIGHT,
        };
        let author_theme = if self.system {
            SYSTEM_CHAT
        } else {
            Style::default().fg(self.hash_username(&frontend_config.palette))
        };
        let datetime_theme = match frontend_config.theme {
            Theme::Dark => DATETIME_DARK,
            _ => DATETIME_LIGHT,
        };
        let search_theme = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);

        // All indices to highlight like a user
        let username_highlight = username_highlight
            .map(|name| {
                self.payload
                    .match_indices(name)
                    .flat_map(move |(index, _)| index + 1..=(index + name.len()))
                    .collect::<Vec<usize>>()
            })
            .unwrap_or_default();

        // All indices to highlight like a search result
        let search_highlight = search_highlight
            .and_then(|query| {
                FUZZY_FINDER
                    .fuzzy_indices(&self.payload, query)
                    .map(|(_, indices)| indices)
            })
            .unwrap_or_default();

        // Message prefix
        let time_sent = self
            .time_sent
            .format(&frontend_config.datetime_format)
            .to_string();

        // Add 2 for the " " and ":"
        let prefix_len = if frontend_config.username_shown {
            time_sent.len() + self.author.len() + 2
        } else {
            time_sent.len()
        };

        // Width of the window - window margin on both sides
        let wrap_limit = {
            // Add 1 for the border line
            let window_margin = usize::from(frontend_config.margin) + 1;
            width - window_margin * 2
        } - 1;

        let prefix = " ".repeat(prefix_len);
        let opts = textwrap::Options::new(wrap_limit).initial_indent(&prefix);
        let wrapped_message = textwrap::wrap(&self.payload, opts);
        if wrapped_message.is_empty() {
            return vec![];
        }
        let mut lines = wrapped_message.into_iter();

        let username_alignment = if frontend_config.username_shown {
            if frontend_config.right_align_usernames {
                NAME_MAX_CHARACTERS - self.author.len() + 1
            } else {
                1
            }
        } else {
            0
        };

        let mut first_row = vec![
            // Datetime
            Span::styled(time_sent, datetime_theme),
            Span::raw(" ".repeat(username_alignment)),
        ];

        if frontend_config.username_shown {
            first_row.extend(vec![
                // Author
                Span::styled(&self.author, author_theme),
                Span::raw(":"),
            ]);
        }

        let mut next_index = 0;

        // Unwrapping is safe because of the empty check above
        let mut first_line = lines.next().unwrap();
        let first_line_msg = split_cow_in_place(&mut first_line, prefix_len - 1);

        first_row.extend(highlight(
            first_line_msg,
            &mut next_index,
            &search_highlight,
            search_theme,
            &username_highlight,
            username_theme,
        ));

        let mut rows = vec![Line::from(first_row)];

        rows.extend(lines.map(|line| {
            Line::from(highlight(
                line,
                &mut next_index,
                &search_highlight,
                search_theme,
                &username_highlight,
                username_theme,
            ))
        }));

        rows
    }

    /// Splits the payload by spaces, then check every word to see if they match an emote.
    /// If they do, tell the terminal to load the emote, and replace the word by `'a' * emote_width`,
    /// which will later be replaced by spaces.
    /// Some emotes can stack on top of each others, in this case we remove the word entirely from the payload.
    pub fn parse_emotes(&mut self, emotes: &mut Emotes) {
        let mut position = 1;
        let mut last_emote_pos = 0;

        self.payload = self
            .payload
            .split(' ')
            .filter_map(|word| {
                if let Some((filename, zero_width)) = emotes.emotes.get(word) {
                    match load_emote(
                        word,
                        filename,
                        *zero_width,
                        &mut emotes.info,
                        &mut emotes.loaded,
                        emotes.cell_size,
                    ) {
                        Ok(loaded_emote) => {
                            if loaded_emote.overlay {
                                // Check if last word is emote.
                                if let Some(emote) = self.emotes.last() {
                                    if last_emote_pos == position {
                                        self.emotes.push(EmoteData::new(
                                            &loaded_emote,
                                            word.to_string(),
                                            emote.index_in_message,
                                            emote.layer + 1,
                                        ));
                                        return None;
                                    }
                                }
                            }
                            self.emotes.push(EmoteData::new(
                                &loaded_emote,
                                word.to_string(),
                                position,
                                0,
                            ));

                            position += loaded_emote.width as usize + 1;
                            last_emote_pos = position;
                            return Some("a".repeat(loaded_emote.width as usize));
                        }
                        Err(err) => {
                            warn!("Unable to load emote {word} ({filename}): {err}");
                            emotes.emotes.remove(word);
                        }
                    }
                }
                position += word.width() + 1;
                Some(word.to_string())
            })
            .collect::<Vec<_>>()
            .join(" ");
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DataBuilder<'conf> {
    pub datetime_format: &'conf str,
}

impl<'conf> DataBuilder<'conf> {
    pub const fn new(datetime_format: &'conf str) -> Self {
        DataBuilder { datetime_format }
    }

    pub fn user(user: String, payload: String, id: Option<String>) -> TwitchToTerminalAction {
        TwitchToTerminalAction::Message(MessageData::new(user, false, payload, id))
    }

    pub fn system(self, payload: String) -> TwitchToTerminalAction {
        TwitchToTerminalAction::Message(MessageData::new("System".to_string(), true, payload, None))
    }

    pub fn twitch(self, payload: String) -> TwitchToTerminalAction {
        TwitchToTerminalAction::Message(MessageData::new("Twitch".to_string(), true, payload, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_hash() {
        assert_eq!(
            MessageData::new("human".to_string(), false, "beep boop".to_string(), None)
                .hash_username(&Palette::Pastel),
            Rgb(159, 223, 221)
        );
    }
}
