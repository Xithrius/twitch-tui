use std::fmt::Display;

use bon::bon;
use color_eyre::Result;
use ratatui_textarea::{CursorMove, DataCursor, TextArea, WrapMode};
use tokio::sync::mpsc::Sender;
use tui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, TitlePosition, Widget},
};
use unicode_segmentation::UnicodeSegmentation;

use super::popup_area;
use crate::{
    config::SharedCoreConfig,
    events::{Event, InternalEvent, Key},
    ui::components::Component,
    utils::{
        styles::NO_COLOR,
        text::{TitleStyle, title_line},
    },
};

pub type InputValidator<T> = Box<dyn Fn(T, String) -> bool>;
pub type VisualValidator = Box<dyn Fn(String) -> String>;
pub type InputSuggester<T> = Box<dyn Fn(T, String) -> Option<String>>;

/// Char column of grapheme index `idx`.
#[expect(
    clippy::indexing_slicing,
    reason = "callers pass idx + 1 or w2_end, which are never past graphemes.len()"
)]
fn char_col(graphemes: &[&str], idx: usize) -> usize {
    graphemes[..idx].iter().map(|g| g.chars().count()).sum()
}

pub struct InputWidget<T: Clone> {
    config: SharedCoreConfig,
    event_tx: Sender<Event>,
    input: TextArea<'static>,
    title: String,
    focused: bool,
    input_validator: Option<(T, InputValidator<T>)>,
    visual_indicator: Option<VisualValidator>,
    input_suggester: Option<(T, InputSuggester<T>)>,
    suggestion: Option<String>,
    max_lines: u16,
}

#[bon]
impl<T: Clone> InputWidget<T> {
    #[builder]
    pub fn new(
        config: SharedCoreConfig,
        event_tx: Sender<Event>,
        title: &str,
        input_validator: Option<(T, InputValidator<T>)>,
        visual_indicator: Option<VisualValidator>,
        input_suggester: Option<(T, InputSuggester<T>)>,
        /// Rows the input may grow to before it scrolls. Above 1, long input soft-wraps
        /// according to `frontend.input_wrap`.
        #[builder(default = 1)]
        max_lines: u16,
    ) -> Self {
        let mut input = TextArea::default();
        input.set_cursor_line_style(Style::default());
        if max_lines > 1 {
            input.set_wrap_mode(config.frontend.input_wrap.clone().into());
        }

        Self {
            config,
            event_tx,
            input,
            title: title.to_string(),
            focused: false,
            input_validator,
            visual_indicator,
            input_suggester,
            suggestion: None,
            max_lines,
        }
    }

    /// Height including borders when drawn `width` columns wide,
    /// growing with the wrapped input up to `max_lines` rows.
    pub fn height(&self, width: u16) -> u16 {
        let inner_width = width.saturating_sub(2);
        if self.max_lines <= 1 || self.input.wrap_mode() == WrapMode::None || inner_width == 0 {
            return 3;
        }

        // The textarea's wrapped row count isn't public, so render a copy with the cursor
        // on the last character and see which wrapped row it lands on.
        let mut probe = self.input.clone();
        probe.move_cursor(CursorMove::Bottom);
        probe.move_cursor(CursorMove::End);
        let area = Rect::new(0, 0, inner_width, 1);
        (&probe).render(area, &mut Buffer::empty(area));

        let rows = probe.screen_cursor().row as u16 + 1;
        rows.clamp(1, self.max_lines) + 2
    }

    pub fn clear(&mut self) {
        self.input.clear();
    }

    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    pub const fn toggle_focus(&mut self) {
        self.focused = !self.focused;
    }

    pub fn toggle_focus_with(&mut self, s: &str) {
        self.focused = !self.focused;
        self.input.clear();
        self.input.insert_str(s);
    }

    pub fn is_valid(&self) -> bool {
        self.input_validator
            .as_ref()
            .is_none_or(|(items, validator)| {
                validator(items.clone(), self.input.lines().join("\n"))
            })
    }

    pub fn accept_suggestion(&mut self) {
        if let Some(suggestion) = &self.suggestion {
            self.input.clear();
            self.input.insert_str(suggestion);
        }
    }

    pub fn insert(&mut self, s: &str) {
        self.input.insert_str(s);
    }

    fn cursor_at_end(&self) -> bool {
        let DataCursor(row, col) = self.input.cursor();
        let lines = self.input.lines();
        row + 1 == lines.len() && lines.get(row).is_some_and(|l| col == l.chars().count())
    }

    /// Graphemes of the cursor's current line, and the cursor as a grapheme index.
    ///
    /// If mid-grapheme, the index is rounded down.
    fn line_graphemes(&self) -> (Vec<&str>, usize) {
        let DataCursor(row, col) = self.input.cursor();
        let graphemes: Vec<&str> = self
            .input
            .lines()
            .get(row)
            .map_or_default(|l| l.graphemes(true).collect());

        let mut chars = 0;
        let idx = graphemes
            .iter()
            .take_while(|g| {
                chars += g.chars().count();
                chars <= col
            })
            .count();

        (graphemes, idx)
    }

    /// Replace the cursor's line with `new` and put the cursor at `col`, keeping the yank buffer intact.
    fn replace_current_line(&mut self, new: &str, col: usize) {
        let row = self.input.cursor().0;
        let yank = self.input.yank_text();

        self.input.move_cursor(CursorMove::Head);
        self.input.delete_line_by_end();
        self.input.insert_str(new);
        self.input
            .move_cursor(CursorMove::Jump(row as u16, col as u16));

        self.input.set_yank_text(yank);
    }

    fn transpose_chars(&mut self) {
        let (mut g, mut idx) = self.line_graphemes();

        if idx == 0 || g.len() < 2 {
            return;
        }
        // At the end of the line, swap the last two graphemes instead
        if idx == g.len() {
            idx -= 1;
        }

        g.swap(idx - 1, idx);
        let col = char_col(&g, idx + 1);
        let new = g.concat();
        self.replace_current_line(&new, col);
    }

    #[expect(
        clippy::indexing_slicing,
        reason = "indexing is guarded by the loop conditions, and slice bounds are ordered w1_beg <= w1_end <= w2_beg <= w2_end <= len"
    )]
    fn transpose_words(&mut self) {
        let (graphemes, idx) = self.line_graphemes();

        let is_word = |s: &str| s.chars().next().is_some_and(char::is_alphanumeric);
        let word_end = |mut i: usize| {
            while i < graphemes.len() && is_word(graphemes[i]) {
                i += 1;
            }
            i
        };
        let prev_word_start = |mut i: usize| {
            while i > 0 && !is_word(graphemes[i - 1]) {
                i -= 1;
            }
            while i > 0 && is_word(graphemes[i - 1]) {
                i -= 1;
            }
            i
        };

        // Second word: the one under/after the cursor, or the last one before it
        let mut i = idx;
        while i < graphemes.len() && !is_word(graphemes[i]) {
            i += 1;
        }
        let w2_beg = prev_word_start(word_end(i));
        let w2_end = word_end(w2_beg);

        // First word: the one before that
        let w1_beg = prev_word_start(w2_beg);
        let w1_end = word_end(w1_beg);

        // No second word, or only punctuation before it (`prev_word_start` then stops at 0 without finding a word)
        if w1_beg == w2_beg || w1_beg == w1_end {
            return;
        }

        let swapped = [
            &graphemes[..w1_beg],
            &graphemes[w2_beg..w2_end],
            &graphemes[w1_end..w2_beg],
            &graphemes[w1_beg..w1_end],
            &graphemes[w2_end..],
        ]
        .concat();

        // Same grapheme count, but char counts per position changed, so compute the column on the new order
        let col = char_col(&swapped, w2_end);
        let new = swapped.concat();
        self.replace_current_line(&new, col);
    }
}

impl<T: Clone> Display for InputWidget<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.input.lines().join("\n"))
    }
}

impl<T: Clone> Component for InputWidget<T> {
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) -> Result<()> {
        let r = area.map_or_else(|| popup_area(f.area(), 60, 60), |a| a.clamp(f.area()));
        let text = self.input.lines().join("\n");

        let status_color = if self.is_valid() {
            Color::Green
        } else {
            Color::Red
        };

        self.suggestion = self
            .input_suggester
            .as_ref()
            .and_then(|(items, suggester)| suggester(items.clone(), text.clone()));

        let binding = [TitleStyle::Single(&self.title)];
        let title_style = if *NO_COLOR {
            Style::default()
        } else {
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(self.config.frontend.border_type.clone().into())
            .border_style(Style::default().fg(status_color))
            .title(title_line(&binding, title_style));
        let inner = block.inner(r);

        f.render_widget(Clear, r);
        f.render_widget(block, r);
        f.render_widget(&self.input, inner);

        if self.cursor_at_end()
            && let Some(rest) = self
                .suggestion
                .as_deref()
                .and_then(|s| s.strip_prefix(text.as_str()))
                .filter(|s| !s.is_empty())
        {
            // Only valid after the render above
            let cursor = self.input.screen_cursor();
            let (top, left) = self.input.scroll_offset();
            let x = inner.x + (cursor.col as u16).saturating_sub(left);
            let y = inner.y + (cursor.row as u16).saturating_sub(top);

            if x < inner.right() && y < inner.bottom() {
                let mut chars = rest.chars();
                let first = chars.next().map_or_default(String::from);
                let line = Line::from(vec![
                    Span::styled(first, self.input.cursor_style().add_modifier(Modifier::DIM)),
                    Span::styled(chars.as_str(), Style::default().add_modifier(Modifier::DIM)),
                ]);
                f.render_widget(Paragraph::new(line), Rect::new(x, y, inner.right() - x, 1));
            }
        }

        if let Some(visual) = &self.visual_indicator {
            let contents = visual(text);

            let title = [TitleStyle::Single(&contents)];

            let bottom_block = Block::default()
                .title(title_line(
                    &title,
                    if *NO_COLOR {
                        Style::default()
                    } else {
                        Style::default()
                            .fg(status_color)
                            .add_modifier(Modifier::BOLD)
                    },
                ))
                .title_position(TitlePosition::Bottom)
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_type(self.config.frontend.border_type.clone().into());

            // This is only supposed to render on the very bottom line of the area.
            // If some rendering breaks for input boxes, this is a possible source.
            let rect = Rect::new(r.x, r.bottom() - 1, r.width, 1);
            f.render_widget(bottom_block, rect);
        }

        Ok(())
    }

    async fn event(&mut self, event: &Event) -> Result<()> {
        if let Event::Input(key) = event {
            let keybinds = &self.config.keybinds.insert;
            match key {
                key if keybinds.move_cursor_right.contains(key) => {
                    if self.cursor_at_end() {
                        self.accept_suggestion();
                        self.input.move_cursor(ratatui_textarea::CursorMove::End);
                    } else {
                        self.input
                            .move_cursor(ratatui_textarea::CursorMove::Forward);
                    }
                }
                key if keybinds.move_cursor_left.contains(key) => {
                    self.input.move_cursor(ratatui_textarea::CursorMove::Back);
                }
                key if keybinds.move_cursor_start.contains(key) => {
                    self.input.move_cursor(ratatui_textarea::CursorMove::Head);
                }
                key if keybinds.move_cursor_end.contains(key) => {
                    self.input.move_cursor(ratatui_textarea::CursorMove::End);
                }
                key if keybinds.end_of_next_word.contains(key) => {
                    self.input
                        .move_cursor(ratatui_textarea::CursorMove::WordEnd);
                }
                key if keybinds.start_of_previous_word.contains(key) => {
                    self.input
                        .move_cursor(ratatui_textarea::CursorMove::WordBack);
                }
                key if keybinds.swap_previous_item_with_current.contains(key) => {
                    self.transpose_chars();
                }
                key if keybinds.swap_previous_word_with_current.contains(key) => {
                    self.transpose_words();
                }
                key if keybinds.remove_before_cursor.contains(key) => {
                    self.input.delete_line_by_head();
                }
                key if keybinds.remove_after_cursor.contains(key) => {
                    self.input.delete_line_by_end();
                }
                key if keybinds.remove_previous_word.contains(key) => {
                    self.input.delete_word();
                }
                key if keybinds.remove_item_to_right.contains(key) => {
                    self.input.delete_next_word();
                }
                Key::Backspace => {
                    self.input.delete_char();
                }
                key if keybinds.fill_suggestion.contains(key) => {
                    if let Some(suggestion) = &self.suggestion {
                        self.input.insert_str(suggestion);
                    }
                }
                key if keybinds.quit.contains(key) => {
                    self.event_tx
                        .send(Event::Internal(InternalEvent::Quit))
                        .await?;
                }
                Key::Char(c) => {
                    self.input.insert_char(*c);
                }
                _ => {}
            }
        } else if let Event::Paste(text) = event {
            // Twitch messages are single-line, so join pasted lines with spaces
            self.input
                .insert_str(text.replace("\r\n", " ").replace(['\r', '\n'], " "));
        }

        Ok(())
    }
}
