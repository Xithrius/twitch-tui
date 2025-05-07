use std::{borrow::Cow, mem::swap, string::ToString, sync::LazyLock};

use chrono::{DateTime, offset::Local};
use fuzzy_matcher::FuzzyMatcher;
use memchr::{memchr_iter, memmem};
use tracing::{error, warn};
use tui::{
    style::{Color, Color::Rgb, Modifier, Style},
    text::{Line, Span},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    emotes::{DownloadedEmotes, EmoteData, SharedEmotes, display_emote, load_emote, overlay_emote},
    handlers::config::{FrontendConfig, Palette, Theme},
    ui::statics::NAME_MAX_CHARACTERS,
    utils::{
        colors::{hsl_to_rgb, u32_to_color},
        emotes::{
            PRIVATE_USE_UNICODE, UnicodePlaceholder, ZERO_WIDTH_SPACE, ZERO_WIDTH_SPACE_STR,
            get_emote_offset,
        },
        search::FUZZY_FINDER,
        styles::{
            DATETIME_DARK_STYLE, DATETIME_LIGHT_STYLE, HIGHLIGHT_NAME_DARK_STYLE,
            HIGHLIGHT_NAME_LIGHT_STYLE, SEARCH_STYLE, SYSTEM_CHAT_STYLE,
        },
        text::split_cow_in_place,
    },
};

pub enum TwitchToTerminalAction {
    Message(RawMessageData),
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
    pub emotes: Vec<(Color, Color)>,
    pub message_id: Option<String>,
    pub highlight: bool,
}

#[derive(Debug, Clone)]
pub struct RawMessageData {
    pub time_sent: DateTime<Local>,
    pub author: String,
    pub user_id: Option<String>,
    pub system: bool,
    pub payload: String,
    pub emotes: DownloadedEmotes,
    pub message_id: Option<String>,
    pub highlight: bool,
}

impl RawMessageData {
    pub fn new(
        author: String,
        user_id: Option<String>,
        system: bool,
        payload: String,
        emotes: DownloadedEmotes,
        message_id: Option<String>,
        highlight: bool,
    ) -> Self {
        Self {
            time_sent: Local::now(),
            author,
            user_id,
            system,
            payload,
            emotes,
            message_id,
            highlight,
        }
    }
}

type Highlight<'a> = (&'a [usize], Style);

impl MessageData {
    /// Create a message and parse its emotes using global emotes, and twitch emotes provided through [`RawMessageData`]
    pub fn from_twitch_message(msg: RawMessageData, emotes: &SharedEmotes) -> Self {
        let (payload, emotes) = Self::parse_emotes(
            msg.payload,
            emotes,
            &msg.emotes,
            &emotes.global_emotes.borrow(),
        );

        Self {
            time_sent: msg.time_sent,
            author: msg.author,
            user_id: msg.user_id,
            system: msg.system,
            payload,
            emotes,
            message_id: msg.message_id,
            highlight: msg.highlight,
        }
    }

    pub fn reparse_emotes(&mut self, emotes: &SharedEmotes) {
        // Small hack to avoid cloning `self.payload`
        let mut payload = String::new();
        swap(&mut payload, &mut self.payload);

        let (payload, emotes) = Self::parse_emotes(
            payload,
            emotes,
            &emotes.global_emotes.borrow(),
            &DownloadedEmotes::default(),
        );

        self.payload = payload;
        self.emotes.extend(emotes);
    }

    fn is_emote<'a>(
        word: &str,
        set1: &'a DownloadedEmotes,
        set2: &'a DownloadedEmotes,
    ) -> Option<(&'a str, bool)> {
        if let Some((filename, zero_width)) = set1.get(word) {
            Some((filename, *zero_width))
        } else if let Some((filename, zero_width)) = set2.get(word) {
            Some((filename, *zero_width))
        } else {
            None
        }
    }

    /// Splits the payload by spaces, then check every word to see if they match an emote.
    /// If they do, tell the terminal to load the emote, and replace the word by a [`UnicodePlaceholder`].
    /// The emote will then be displayed by the terminal by encoding its id in its foreground color, and its pid in its underline color.
    /// Ratatui removes all ansi escape sequences, so the id/pid of the emote is stored and encoded in [`MessageData::to_vec`].
    fn parse_emotes(
        mut payload: String,
        emotes: &SharedEmotes,
        emotes_set1: &DownloadedEmotes,
        emotes_set2: &DownloadedEmotes,
    ) -> (String, Vec<(Color, Color)>) {
        if !emotes.enabled {
            return (payload, vec![]);
        }

        let cell_size = *emotes
            .cell_size
            .get()
            .expect("Terminal cell_size must be defined when emotes are enabled");

        if emotes_set1.is_empty() && emotes_set2.is_empty() {
            return (payload, vec![]);
        }

        let mut words = Vec::new();

        payload.split([' ', ZERO_WIDTH_SPACE]).for_each(|word| {
            let Some((filename, zero_width)) = Self::is_emote(word, emotes_set1, emotes_set2)
            else {
                words.push(Word::Text(word.to_string()));
                return;
            };

            let Ok(loaded_emote) = load_emote(
                word,
                filename,
                zero_width,
                &mut emotes.info.borrow_mut(),
                cell_size,
            )
            .map_err(|e| warn!("Unable to load emote {word} ({filename}): {e}")) else {
                words.push(Word::Text(word.to_string()));
                return;
            };

            if loaded_emote.overlay {
                // Check if last word is an emote.
                if let Some(Word::Emote(v)) = words.last_mut() {
                    v.push(loaded_emote.into());
                    return;
                }
            }

            words.push(Word::Emote(vec![loaded_emote.into()]));
        });

        payload.clear();

        let mut emotes = vec![];
        // Join words by space, or by zero-width spaces if one of them is an emote.
        for w in words {
            match w {
                Word::Text(s) => {
                    if !payload.is_empty() {
                        payload.push(if payload.ends_with(PRIVATE_USE_UNICODE) {
                            ZERO_WIDTH_SPACE
                        } else {
                            ' '
                        });
                    }
                    payload.push_str(&s);
                }
                Word::Emote(v) => {
                    // Unwrapping here is fine as v is never empty.
                    let max_width = v
                        .iter()
                        .max_by_key(|e| e.width)
                        .expect("Emotes should never be empty")
                        .width as f32;
                    let cols = (max_width / cell_size.0).ceil() as u16;

                    let mut iter = v.into_iter();

                    let EmoteData { id, pid, width } = iter.next().unwrap();

                    let (_, col_offset) = get_emote_offset(width as u16, cell_size.0 as u16, cols);

                    if let Err(e) = display_emote(id, pid, cols) {
                        warn!("Unable to display emote: {e}");
                        continue;
                    }

                    iter.enumerate().for_each(|(layer, emote)| {
                        if let Err(e) = overlay_emote(
                            (id, pid),
                            emote,
                            layer as u32,
                            cols,
                            col_offset,
                            cell_size.0 as u16,
                        ) {
                            warn!("Unable to display overlay: {e}");
                        }
                    });

                    emotes.push((u32_to_color(id), u32_to_color(pid)));

                    if !payload.is_empty() {
                        payload.push(ZERO_WIDTH_SPACE);
                    }

                    payload.extend(UnicodePlaceholder::new(cols as usize).iter());
                }
            }
        }

        (payload, emotes)
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

    fn char_to_byte_indices(s: &str, char_indices: impl Iterator<Item = usize>) -> Vec<usize> {
        let mut chars = s.chars();
        let mut positions = 0..;

        char_indices
            .filter_map(|index| {
                while let (Some(_), Some(p)) = (chars.next(), positions.next()) {
                    if index == p {
                        return Some(s.len() - 1 - chars.as_str().len());
                    }
                }
                None
            })
            .collect()
    }

    fn get_emote_span<'s>(
        content: impl Into<Cow<'s, str>>,
        emotes: &mut &[(Color, Color)],
    ) -> Span<'s> {
        #[allow(unused_variables)]
        if let Some(&(id, pid)) = emotes.first() {
            *emotes = &emotes[1..];

            #[cfg(not(target_os = "windows"))]
            {
                Span::styled(content, Style::default().fg(id).underline_color(pid))
            }

            #[cfg(target_os = "windows")]
            {
                Span::styled(content, Style::default().fg(id))
            }
        } else {
            error!("Emote index >= emotes.len()");
            Span::raw(content)
        }
    }

    fn highlight<'s>(
        line: Cow<'s, str>,
        start_index: &mut usize,
        default_style: Style,
        (search_highlight, search_theme): Highlight,
        (username_highlight, username_theme): Highlight,
    ) -> Vec<Span<'s>> {
        const HAS_NO_HIGHLIGHTS: fn(&[usize], &usize, &usize) -> bool =
            |highlight: &[usize], start: &usize, end: &usize| {
                highlight.is_empty()
                    || start > highlight.last().unwrap()
                    || end < highlight.first().unwrap()
            };

        let offset = *start_index;
        *start_index += line.len();

        if HAS_NO_HIGHLIGHTS(search_highlight, &offset, start_index)
            && HAS_NO_HIGHLIGHTS(username_highlight, &offset, start_index)
        {
            return vec![Span::styled(line, default_style)];
        }

        line.char_indices()
            .map(|(i, c)| (i + offset, c))
            .map(|(i, c)| {
                if search_highlight.binary_search(&i).is_ok() {
                    Span::styled(c.to_string(), search_theme)
                } else if username_highlight.binary_search(&i).is_ok() {
                    Span::styled(c.to_string(), username_theme)
                } else {
                    Span::styled(c.to_string(), default_style)
                }
            })
            .collect()
    }

    fn build_line<'s>(
        line: Cow<'s, str>,
        start_index: &mut usize,
        default_style: Style,
        search_highlight: Highlight,
        username_highlight: Highlight,
        emotes: &mut &[(Color, Color)],
    ) -> Vec<Span<'s>> {
        static EMOTE_FINDER: LazyLock<memmem::Finder> =
            LazyLock::new(|| memmem::Finder::new(ZERO_WIDTH_SPACE_STR));

        // A line contains emotes if `emotes` is not empty and `line` starts with a unicode placeholder or contains ZWS.
        if emotes.is_empty()
            || (!line.starts_with(PRIVATE_USE_UNICODE)
                && EMOTE_FINDER.find(line.as_bytes()).is_none())
        {
            Self::highlight(
                line,
                start_index,
                default_style,
                search_highlight,
                username_highlight,
            )
        } else {
            let mut spans: Vec<Span<'s>> = vec![];

            for s in match line {
                Cow::Borrowed(b) => Box::new(b.split(ZERO_WIDTH_SPACE).map(Cow::Borrowed))
                    as Box<dyn Iterator<Item = Cow<'s, str>>>,
                Cow::Owned(ref o) => {
                    Box::new(o.split(ZERO_WIDTH_SPACE).map(String::from).map(Cow::Owned))
                        as Box<dyn Iterator<Item = Cow<'s, str>>>
                }
            } {
                if s.starts_with(PRIVATE_USE_UNICODE) {
                    *start_index += s.len();
                    spans.push(Self::get_emote_span(s, emotes));
                } else {
                    spans.extend(Self::highlight(
                        s,
                        start_index,
                        default_style,
                        search_highlight,
                        username_highlight,
                    ));
                }
                *start_index += ZERO_WIDTH_SPACE_STR.len();
            }

            *start_index = start_index.saturating_sub(ZERO_WIDTH_SPACE_STR.len());
            spans
        }
    }

    pub fn to_vec(
        &self,
        frontend_config: &FrontendConfig,
        width: usize,
        search_highlight: Option<&str>,
        username_highlight: Option<&str>,
    ) -> Vec<Line> {
        // Theme styles
        let fg = self.hash_username(&frontend_config.palette);

        let message_theme = if self.highlight {
            Style::default().fg(fg).add_modifier(Modifier::ITALIC)
        } else {
            Style::default()
        };
        let username_theme = match frontend_config.theme {
            Theme::Dark => *HIGHLIGHT_NAME_DARK_STYLE,
            _ => *HIGHLIGHT_NAME_LIGHT_STYLE,
        };
        let author_theme = if self.system {
            *SYSTEM_CHAT_STYLE
        } else {
            Style::default().fg(fg)
        };
        let datetime_theme = match frontend_config.theme {
            Theme::Dark => *DATETIME_DARK_STYLE,
            _ => *DATETIME_LIGHT_STYLE,
        };

        // All indices to highlight like a user
        let username_highlight = username_highlight
            .map(|name| {
                self.payload
                    .match_indices(name)
                    .flat_map(|(index, _)| index..(index + name.len()))
                    .collect::<Vec<usize>>()
            })
            .unwrap_or_default();

        // All indices to highlight like a search result
        let search_highlight = search_highlight
            .and_then(|query| {
                FUZZY_FINDER
                    .fuzzy_indices(&self.payload, query)
                    .map(|(_, indices)| {
                        // `username_highlight` indices are byte indices, whereas `fuzzy_indices` returns char indices.
                        // Convert those char indices to byte indices, which are easier to work with.
                        Self::char_to_byte_indices(&self.payload, indices.into_iter())
                    })
            })
            .unwrap_or_default();

        let search = (&search_highlight as &[usize], *SEARCH_STYLE);
        let username = (&username_highlight as &[usize], username_theme);

        // Message prefix
        let time_sent = if frontend_config.show_datetimes {
            Some(
                self.time_sent
                    .format(&frontend_config.datetime_format)
                    .to_string(),
            )
        } else {
            None
        };

        // Add 1 for the space after the timestamp
        let time_sent_len = time_sent.as_ref().map_or(0, |t| t.len() + 1);

        let prefix_len = if frontend_config.username_shown {
            // Add 2 for the ": "
            time_sent_len + self.author.len() + 2
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
                NAME_MAX_CHARACTERS.saturating_sub(self.author.width()) + 1
            } else {
                1
            }
        } else {
            1
        };

        let mut first_row: Vec<Span<'_>> = vec![];

        if let Some(t) = time_sent {
            first_row.extend(vec![
                Span::styled(t, datetime_theme),
                Span::raw(" ".repeat(username_alignment)),
            ]);
        }

        if frontend_config.username_shown {
            first_row.extend(vec![
                Span::styled(&self.author, author_theme),
                Span::raw(": "),
            ]);
        }

        let mut next_index = 0;

        // Unwrapping is safe because of the empty check above
        let mut first_line = lines.next().unwrap();
        let first_line_msg = split_cow_in_place(&mut first_line, prefix_len);

        // Find all spaces in `self.payload`, to increment `next_index` when a space at the end of a line
        // is encountered. Wrapping the message above removes spaces at eol, which makes the highlight indices off by 1.
        let mut space_iter = memchr_iter(b' ', self.payload.as_bytes());

        let mut emotes = &self.emotes[..];

        first_row.extend(Self::build_line(
            first_line_msg,
            &mut next_index,
            message_theme,
            search,
            username,
            &mut emotes,
        ));

        let mut space_idx = space_iter.next();

        let mut rows = vec![Line::from(first_row)];

        rows.extend(lines.map(|line| {
            while space_idx.is_some_and(|x| x < next_index) {
                space_idx = space_iter.next();
            }

            if space_idx == Some(next_index) {
                next_index += 1;
                space_idx = space_iter.next();
            }

            Line::from(Self::build_line(
                line,
                &mut next_index,
                message_theme,
                search,
                username,
                &mut emotes,
            ))
        }));

        rows
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DataBuilder;

impl DataBuilder {
    // User messages that come from either twitch or the terminal
    pub fn user(
        user: String,
        user_id: Option<String>,
        payload: String,
        emotes: DownloadedEmotes,
        message_id: Option<String>,
        highlight: bool,
    ) -> TwitchToTerminalAction {
        TwitchToTerminalAction::Message(RawMessageData::new(
            user, user_id, false, payload, emotes, message_id, highlight,
        ))
    }

    /// Notification messages from the terminal
    pub fn system(payload: String) -> TwitchToTerminalAction {
        TwitchToTerminalAction::Message(RawMessageData::new(
            "System".to_string(),
            None,
            true,
            payload,
            DownloadedEmotes::default(),
            None,
            false,
        ))
    }

    /// Notification messages from Twitch
    pub fn twitch(payload: String) -> TwitchToTerminalAction {
        TwitchToTerminalAction::Message(RawMessageData::new(
            "Twitch".to_string(),
            None,
            true,
            payload,
            DownloadedEmotes::default(),
            None,
            false,
        ))
    }
}

#[cfg(test)]
#[cfg(not(target_os = "windows"))]
mod tests {
    use std::{collections::BTreeMap, rc::Rc};

    use crate::emotes::Emotes;

    use super::*;

    #[test]
    fn test_username_hash() {
        assert_eq!(
            MessageData {
                time_sent: DateTime::default(),
                author: "human".to_string(),
                user_id: None,
                system: false,
                payload: "beep boop".to_string(),
                emotes: vec![],
                message_id: None,
                highlight: false,
            }
            .hash_username(&Palette::Pastel),
            Rgb(159, 223, 221)
        );
    }

    const EMOTES_ID_PID: [(Color, Color); 3] = [
        (Color::Red, Color::Green),
        (Color::Black, Color::Cyan),
        (Color::Yellow, Color::Blue),
    ];

    macro_rules! id_pid_to_style {
        ($x:expr) => {
            Style::new().fg($x.0).underline_color($x.1)
        };
    }

    const STYLES: [Style; 3] = [
        id_pid_to_style!(EMOTES_ID_PID[0]),
        id_pid_to_style!(EMOTES_ID_PID[1]),
        id_pid_to_style!(EMOTES_ID_PID[2]),
    ];

    const NO_HIGHLIGHTS: (&[usize], Style) = ([].as_slice(), Style::new());

    #[test]
    fn emote_span() {
        let mut emotes = EMOTES_ID_PID.as_slice();

        let span = MessageData::get_emote_span("", &mut emotes);
        assert_eq!(emotes.len(), 2);
        assert_eq!(span, Span::styled("", STYLES[0]));

        let span = MessageData::get_emote_span("", &mut emotes);
        assert_eq!(emotes.len(), 1);
        assert_eq!(span, Span::styled("", STYLES[1]));

        let span = MessageData::get_emote_span("", &mut emotes);
        assert!(emotes.is_empty());
        assert_eq!(span, Span::styled("", STYLES[2]));
    }

    #[test]
    fn highlight_line() {
        let line = Cow::Borrowed("foo bar baz");
        let line_len = line.len();

        let mut start_index = 1;

        let search_highlight = [6, 10].as_slice();
        let username_highlight = [3, 4, 5, 6, 7].as_slice();

        let spans = MessageData::highlight(
            line,
            &mut start_index,
            Style::default(),
            (search_highlight, STYLES[0]),
            (username_highlight, STYLES[1]),
        );

        assert_eq!(start_index, line_len + 1);

        assert_eq!(
            spans,
            vec![
                Span::raw("f"),
                Span::raw("o"),
                Span::styled("o", STYLES[1]),
                Span::styled(" ", STYLES[1]),
                Span::styled("b", STYLES[1]),
                Span::styled("a", STYLES[0]),
                Span::styled("r", STYLES[1]),
                Span::raw(" "),
                Span::raw("b"),
                Span::styled("a", STYLES[0]),
                Span::raw("z"),
            ]
        );
    }

    fn assert_build_line<'s>(
        line: &'s str,
        start_index: usize,
        search_highlight: Highlight,
        username_highlight: Highlight,
        emotes: &'s [(Color, Color)],
    ) -> (Vec<Span<'s>>, usize, &'s [(Color, Color)]) {
        // Test with `Cow::Owned`
        let (s1, si1, e1) = {
            let mut emotes = emotes;
            let mut start_index = start_index;
            let spans = MessageData::build_line(
                Cow::Owned(line.to_owned()),
                &mut start_index,
                Style::default(),
                search_highlight,
                username_highlight,
                &mut emotes,
            );

            (spans, start_index, emotes)
        };

        // Test with `Cow::Borrowed`
        let (s2, si2, e2) = {
            let mut emotes = emotes;
            let mut start_index = start_index;
            let spans = MessageData::build_line(
                Cow::Borrowed(line),
                &mut start_index,
                Style::default(),
                search_highlight,
                username_highlight,
                &mut emotes,
            );

            (spans, start_index, emotes)
        };

        assert_eq!(s1, s2);
        assert_eq!(si1, si2);
        assert_eq!(e1, e2);

        (s1, si1, e1)
    }

    #[test]
    fn build_line() {
        let line = "foo bar baz";

        let (spans, _, emotes) = assert_build_line(
            line,
            0,
            NO_HIGHLIGHTS,
            NO_HIGHLIGHTS,
            EMOTES_ID_PID.as_slice(),
        );

        assert_eq!(emotes, EMOTES_ID_PID);

        assert_eq!(spans, vec![Span::raw(line)]);
    }

    #[test]
    fn build_line_with_emotes() {
        let emote_w_1 = UnicodePlaceholder::new(1).string();
        let emote_w_2 = UnicodePlaceholder::new(2).string();
        let emote_w_3 = UnicodePlaceholder::new(3).string();

        let line = format!(
            "foo{ZERO_WIDTH_SPACE}{emote_w_3}{ZERO_WIDTH_SPACE}{emote_w_1}{ZERO_WIDTH_SPACE}bar baz{ZERO_WIDTH_SPACE}{emote_w_2}"
        );

        let (spans, _, emotes) = assert_build_line(
            &line,
            0,
            NO_HIGHLIGHTS,
            NO_HIGHLIGHTS,
            EMOTES_ID_PID.as_slice(),
        );

        assert!(emotes.is_empty());
        assert_eq!(
            spans,
            vec![
                Span::raw("foo"),
                Span::styled(emote_w_3, STYLES[0]),
                Span::styled(emote_w_1, STYLES[1]),
                Span::raw("bar baz"),
                Span::styled(emote_w_2, STYLES[2]),
            ]
        );
    }

    #[test]
    fn build_line_with_highlights() {
        let line = "foo bar baz";

        let search_highlight = [5, 9].as_slice();
        let username_highlight = [2, 3, 4, 5, 6].as_slice();

        let (spans, start_index, emotes) = assert_build_line(
            line,
            0,
            (search_highlight, STYLES[0]),
            (username_highlight, STYLES[1]),
            EMOTES_ID_PID.as_slice(),
        );

        assert_eq!(emotes, EMOTES_ID_PID);

        assert_eq!(start_index, line.len());

        assert_eq!(
            spans,
            vec![
                Span::raw("f"),
                Span::raw("o"),
                Span::styled("o", STYLES[1]),
                Span::styled(" ", STYLES[1]),
                Span::styled("b", STYLES[1]),
                Span::styled("a", STYLES[0]),
                Span::styled("r", STYLES[1]),
                Span::raw(" "),
                Span::raw("b"),
                Span::styled("a", STYLES[0]),
                Span::raw("z"),
            ]
        );
    }

    #[test]
    fn build_line_with_emotes_and_highlights() {
        let emote_w_1 = UnicodePlaceholder::new(1).string();
        let emote_w_2 = UnicodePlaceholder::new(2).string();
        let emote_w_3 = UnicodePlaceholder::new(3).string();

        // Line corresponds to "foo{EMOTE3}{EMOTE1}bar {EMOJI} baz{EMOTE2}".
        let line = format!(
            "foo{ZERO_WIDTH_SPACE}{emote_w_3}{ZERO_WIDTH_SPACE}{emote_w_1}{ZERO_WIDTH_SPACE}bar \u{1f7ea} baz{ZERO_WIDTH_SPACE}{emote_w_2}"
        );

        // "ba" in "bar" will be highlighted with the username highlight.
        // " b" in " baz" will be highlighted with the search highlight.
        // "a" in "baz" will be highlighted with the username highlight.
        let username_highlight = line
            .match_indices("ba")
            .flat_map(|(index, _)| index..(index + 2))
            .collect::<Vec<usize>>();

        let search_highlight = FUZZY_FINDER
            .fuzzy_indices(&line, " b")
            .map_or(Vec::new(), |(_, indices)| {
                MessageData::char_to_byte_indices(&line, indices.into_iter())
            });

        let (spans, start_index, emotes) = assert_build_line(
            &line,
            0,
            (search_highlight.as_slice(), STYLES[0]),
            (username_highlight.as_slice(), STYLES[1]),
            EMOTES_ID_PID.as_slice(),
        );

        assert!(emotes.is_empty());
        assert_eq!(start_index, line.len());

        assert_eq!(
            spans,
            vec![
                Span::raw("foo"),
                Span::styled(emote_w_3, STYLES[0]),
                Span::styled(emote_w_1, STYLES[1]),
                Span::styled("b", STYLES[1]),
                Span::styled("a", STYLES[1]),
                Span::raw("r"),
                Span::raw(" "),
                Span::raw("\u{1f7ea}"),
                Span::styled(" ", STYLES[0]),
                Span::styled("b", STYLES[0]),
                Span::styled("a", STYLES[1]),
                Span::raw("z"),
                Span::styled(emote_w_2, STYLES[2]),
            ]
        );
    }

    #[test]
    fn build_vec_with_wraps_and_highlights() {
        let raw_message = RawMessageData::new(
            "foo".to_string(),
            None,
            false,
            "foo bar baz".to_string(),
            BTreeMap::new(),
            None,
            false,
        );

        let emotes = Rc::new(Emotes::new(false));
        let data = MessageData::from_twitch_message(raw_message, &emotes);

        let frontendconfig = FrontendConfig {
            show_datetimes: false,
            ..FrontendConfig::default()
        };

        let lines = data.to_vec(&frontendconfig, 13, Some("bar"), None);

        assert_eq!(
            lines,
            vec![
                Line::from(vec![
                    Span::styled("foo", data.hash_username(&Palette::Pastel)),
                    Span::raw(": "),
                    Span::raw("foo")
                ]),
                Line::from(vec![
                    Span::styled(
                        "b",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                    ),
                    Span::styled(
                        "a",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                    ),
                    Span::styled(
                        "r",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                    ),
                    Span::raw(" "),
                    Span::raw("b"),
                    Span::raw("a"),
                    Span::raw("z")
                ])
            ]
        );
    }
}
