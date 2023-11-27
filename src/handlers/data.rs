use std::{borrow::Cow, iter, string::ToString};

use chrono::{offset::Local, DateTime};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use log::{error, warn};
use once_cell::sync::Lazy;
use tui::{
    style::{Color, Color::Rgb, Modifier, Style},
    text::{Line, Span},
};

use crate::emotes::{display_emote, overlay_emote, EmoteData};
use crate::handlers::data::Word::{Emote, Text};
use crate::{
    emotes::{load_emote, Emotes},
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

const PRIVATE_USE_UNICODE: char = '\u{10EEEE}';
const ZERO_WIDTH_SPACE: char = '\u{200B}';

pub enum TwitchToTerminalAction {
    Message(MessageData),
    ClearChat(Option<String>),
    DeleteMessage(String),
}

enum Word {
    Emote(Vec<EmoteData>),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct MessageData {
    pub time_sent: DateTime<Local>,
    pub author: String,
    pub user_id: Option<String>,
    pub system: bool,
    pub payload: String,
    pub emotes: Vec<(u16, Color, Color)>,
    pub message_id: Option<String>,
}

impl MessageData {
    pub fn new(
        author: String,
        user_id: Option<String>,
        system: bool,
        payload: String,
        message_id: Option<String>,
    ) -> Self {
        Self {
            time_sent: Local::now(),
            author,
            user_id,
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
        type Highlight<'a> = (&'a [usize], Style);

        #[inline]
        fn highlight<'s>(
            line: Cow<'s, str>,
            start_index: &mut usize,
            (search_highlight, search_theme): Highlight,
            (username_highlight, username_theme): Highlight,
            emotes: &[(u16, Color, Color)],
            emotes_idx: &mut usize,
        ) -> Vec<Span<'s>> {
            let get_emote_span = |idx: &mut usize| -> Span {
                if let Some(&(width, id, pid)) = emotes.get(*idx) {
                    *idx += 1;
                    Span::styled(
                        format!(
                            "\u{10EEEE}\u{0305}{}",
                            "\u{10EEEE}".repeat(width as usize - 1)
                        ),
                        Style::default().fg(id).underline_color(pid),
                    )
                } else {
                    error!("Emote index >= emotes.len()");
                    Span::raw("")
                }
            };

            let mut is_emote = false;

            // Fast path
            if search_highlight.is_empty() && username_highlight.is_empty() {
                if !line.contains('\u{10EEEE}') {
                    return vec![Span::raw(line)];
                }

                // Slower path, nothing is highlighted, but message contains emotes.
                let mut spans = vec![];
                line.split('\u{10EEEE}').for_each(|x| {
                    if x.is_empty() {
                        is_emote = true;
                    } else {
                        if is_emote {
                            spans.push(get_emote_span(emotes_idx));
                        }

                        spans.push(Span::raw(x.to_owned()));
                        is_emote = false;
                    }
                });

                if is_emote {
                    spans.push(get_emote_span(emotes_idx));
                }

                return spans;
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
                    } else if c == '\u{10EEEE}' {
                        // This is an emote, only create a span for the first unicode char, skip the rest
                        if is_emote {
                            Span::raw("")
                        } else {
                            is_emote = true;
                            get_emote_span(emotes_idx)
                        }
                    } else {
                        is_emote = false;
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

        let search = (&search_highlight as &[usize], search_theme);
        let username = (&username_highlight as &[usize], username_theme);

        // Message prefix
        let time_sent = self
            .time_sent
            .format(&frontend_config.datetime_format)
            .to_string();

        let time_sent_len = if frontend_config.show_datetimes {
            // Add 1 for the space after the timestamp
            time_sent.len() + 1
        } else {
            0
        };

        let prefix_len = if frontend_config.username_shown {
            // Add 1 for the ":"
            time_sent_len + self.author.len() + 1
        } else {
            time_sent_len
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

        let mut first_row: Vec<Span<'_>> = vec![];

        if frontend_config.show_datetimes {
            first_row.extend(vec![
                Span::styled(time_sent, datetime_theme),
                Span::raw(" ".repeat(username_alignment)),
            ]);
        }

        if frontend_config.username_shown {
            first_row.extend(vec![
                Span::styled(&self.author, author_theme),
                Span::raw(":"),
            ]);
        }

        let mut next_index = 0;

        // Unwrapping is safe because of the empty check above
        let mut first_line = lines.next().unwrap();
        let first_line_msg = split_cow_in_place(&mut first_line, prefix_len - 1);

        let mut emote_idx = 0;
        first_row.extend(highlight(
            first_line_msg,
            &mut next_index,
            search,
            username,
            &self.emotes,
            &mut emote_idx,
        ));

        let mut rows = vec![Line::from(first_row)];

        rows.extend(lines.map(|line| {
            Line::from(highlight(
                line,
                &mut next_index,
                search,
                username,
                &self.emotes,
                &mut emote_idx,
            ))
        }));

        rows
    }

    /// Splits the payload by spaces, then check every word to see if they match an emote.
    /// If they do, tell the terminal to load the emote, and replace the word by `'\u{10EEEE}' * emote_width`.
    /// The emote will then be displayed by the terminal by encoding its id in its foreground color, and its pid in its underline color.
    /// The tui escapes all ansi escape sequences, so the id/color of the emote is stored and encoded in [`MessageData::to_vec`].
    pub fn parse_emotes(&mut self, emotes: &mut Emotes) {
        let mut words = Vec::new();

        self.payload.split(' ').for_each(|word| {
            if let Some((filename, zero_width)) = emotes.emotes.get(word) {
                match load_emote(
                    word,
                    filename,
                    *zero_width,
                    &mut emotes.info,
                    emotes.cell_size,
                ) {
                    Ok(loaded_emote) => {
                        if loaded_emote.overlay {
                            // Check if last word is emote.
                            if let Some(Emote(v)) = words.last_mut() {
                                v.push(loaded_emote.into());
                                return;
                            }
                        }

                        words.push(Emote(vec![loaded_emote.into()]));
                        return;
                    }
                    Err(err) => {
                        warn!("Unable to load emote {word} ({filename}): {err}");
                        emotes.emotes.remove(word);
                    }
                }
            }
            words.push(Text(word.to_string()));
        });

        let words = words
            .into_iter()
            .filter_map(|w| match w {
                Text(s) => Some(s),
                Emote(v) => {
                    let max_width = v.iter().max_by_key(|e| e.width)?.width as f32;
                    let cols = (max_width / emotes.cell_size.0).ceil() as u16;

                    let &EmoteData { id, pid, width } = v.first()?;
                    if let Err(e) = display_emote(id, pid, cols) {
                        warn!("Unable to display emote: {e}");
                        return None;
                    }

                    v.iter().enumerate().skip(1).for_each(|(layer, emote)| {
                        if let Err(e) =
                            overlay_emote((id, pid), emote, layer as u32, width, emotes.cell_size.0)
                        {
                            warn!("Unable to display overlay: {e}");
                        }
                    });

                    let to_rgb = |i: u32| Rgb((i >> 16) as u8, (i >> 8) as u8, i as u8);

                    self.emotes.push((cols, to_rgb(id), to_rgb(pid)));

                    Some(
                        iter::repeat(PRIVATE_USE_UNICODE)
                            .take(cols as usize)
                            .collect(),
                    )
                }
            })
            .collect::<Vec<_>>();

        // Join words by space, skipping spaces before and after emotes
        self.payload.clear();
        let mut iter = words.iter();
        match iter.next() {
            Some(first) => {
                let size = words.iter().map(String::len).sum::<usize>() + words.len() - 1;
                self.payload.reserve(size);
                self.payload.push_str(first);
            }
            None => return,
        }
        for w in iter {
            if !w.starts_with(PRIVATE_USE_UNICODE) && !self.payload.ends_with(PRIVATE_USE_UNICODE) {
                self.payload.push(' ');
            } else {
                self.payload.push(ZERO_WIDTH_SPACE);
            }
            self.payload.push_str(w);
        }
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

    pub fn user(
        user: String,
        user_id: Option<String>,
        payload: String,
        message_id: Option<String>,
    ) -> TwitchToTerminalAction {
        TwitchToTerminalAction::Message(MessageData::new(user, user_id, false, payload, message_id))
    }

    pub fn system(self, payload: String) -> TwitchToTerminalAction {
        TwitchToTerminalAction::Message(MessageData::new(
            "System".to_string(),
            None,
            true,
            payload,
            None,
        ))
    }

    pub fn twitch(self, payload: String) -> TwitchToTerminalAction {
        TwitchToTerminalAction::Message(MessageData::new(
            "Twitch".to_string(),
            None,
            true,
            payload,
            None,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_hash() {
        assert_eq!(
            MessageData::new(
                "human".to_string(),
                None,
                false,
                "beep boop".to_string(),
                None
            )
            .hash_username(&Palette::Pastel),
            Rgb(159, 223, 221)
        );
    }
}
