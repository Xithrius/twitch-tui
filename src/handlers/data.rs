use std::{borrow::Cow, string::ToString};

use chrono::{offset::Local, DateTime};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use lazy_static::lazy_static;
use log::warn;
use tui::{
    style::{Color, Color::Rgb, Modifier, Style},
    text::{Span, Spans},
};
use unicode_width::UnicodeWidthStr;

use crate::emotes::{load_emote, Emotes, LoadedEmote};
use crate::{
    handlers::config::{FrontendConfig, Palette, Theme},
    utils::{
        colors::hsl_to_rgb,
        styles::{
            DATETIME_DARK, DATETIME_LIGHT, HIGHLIGHT_NAME_DARK, HIGHLIGHT_NAME_LIGHT, SYSTEM_CHAT,
        },
        text::split_cow_in_place,
    },
};

lazy_static! {
    pub static ref FUZZY_FINDER: SkimMatcherV2 = SkimMatcherV2::default();
}

#[derive(Debug, Clone)]
pub struct EmoteData {
    pub name: String,
    pub index_in_message: usize,
    pub id: u32,
    pub pid: u32,
}

#[derive(Debug, Clone)]
pub struct MessageData {
    pub time_sent: DateTime<Local>,
    pub author: String,
    pub system: bool,
    pub payload: String,
    pub emotes: Vec<EmoteData>,
}

impl MessageData {
    pub fn new(author: String, system: bool, payload: String) -> Self {
        Self {
            time_sent: Local::now(),
            author,
            system,
            payload,
            emotes: vec![],
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

    pub fn to_spans(
        &self,
        frontend_config: &FrontendConfig,
        width: usize,
        search_highlight: Option<&str>,
        username_highlight: Option<&str>,
    ) -> Vec<Spans> {
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
            let spans = line
                .chars()
                .zip(*start_index..)
                .map(|(c, i)| {
                    if search_highlight.binary_search(&i).is_ok() {
                        Span::styled(c.to_string(), search_theme)
                    } else if username_highlight.binary_search(&i).is_ok() {
                        Span::styled(c.to_string(), username_theme)
                    } else {
                        Span::raw(c.to_string())
                    }
                })
                .collect();
            *start_index += line.len();
            spans
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
                    .flat_map(move |(index, _)| index..(index + name.len()))
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
            .format(&frontend_config.date_format)
            .to_string();

        // Add 2 for the " " and ":"
        let prefix_len = time_sent.len() + self.author.len() + 2;

        // Width of the window - window margin on both sides
        let wrap_limit = {
            // Add 1 for the border line
            let window_margin = usize::from(frontend_config.margin) + 1;
            width - window_margin * 2
        };

        let prefix = " ".repeat(prefix_len);
        let opts = textwrap::Options::new(wrap_limit).initial_indent(&prefix);
        let wrapped_message = textwrap::wrap(&self.payload, opts);
        if wrapped_message.is_empty() {
            return vec![];
        }
        let mut lines = wrapped_message.into_iter();

        let mut first_row = vec![
            // Datetime
            Span::styled(time_sent, datetime_theme),
            Span::raw(" "),
            // Author
            Span::styled(&self.author, author_theme),
            Span::raw(":"),
        ];

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

        let mut rows = vec![Spans(first_row)];

        rows.extend(lines.map(|line| {
            Spans(highlight(
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

    pub fn parse_emotes(&mut self, emotes: &mut Emotes) {
        let mut words: Vec<String> = self.payload.split(' ').map(ToString::to_string).collect();

        let mut position = 1;
        for word in &mut words {
            if let Some(filename) = emotes.emotes.get(word) {
                match load_emote(
                    word,
                    filename,
                    &mut emotes.info,
                    &mut emotes.loaded,
                    emotes.cell_size,
                ) {
                    Ok(LoadedEmote { hash, n, width, .. }) => {
                        self.emotes.push(EmoteData {
                            name: word.clone(),
                            index_in_message: position,
                            id: hash,
                            pid: n,
                        });
                        *word = "a".repeat(width as usize);
                    }
                    Err(err) => {
                        warn!("Unable to load emote {word} ({filename}): {err}");
                        emotes.emotes.remove(word);
                    }
                }
            }
            position += word.width() + 1;
        }
        self.payload = words.join(" ");
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DataBuilder<'conf> {
    pub date_format: &'conf str,
}

impl<'conf> DataBuilder<'conf> {
    pub const fn new(date_format: &'conf str) -> Self {
        DataBuilder { date_format }
    }

    pub fn user(user: String, payload: String) -> MessageData {
        MessageData::new(user, false, payload)
    }

    pub fn system(self, payload: String) -> MessageData {
        MessageData::new("System".to_string(), true, payload)
    }

    pub fn twitch(self, payload: String) -> MessageData {
        MessageData::new("Twitch".to_string(), true, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_hash() {
        assert_eq!(
            MessageData::new("human".to_string(), false, "beep boop".to_string())
                .hash_username(&Palette::Pastel),
            Rgb(159, 223, 221)
        );
    }
}
