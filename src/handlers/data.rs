use chrono::{offset::Local, DateTime};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use lazy_static::lazy_static;
use regex::Regex;
use tui::{
    style::{Color, Color::Rgb, Modifier, Style},
    text::{Span, Spans},
    widgets::{Cell, Row},
};

use crate::{
    handlers::config::{FrontendConfig, Palette, Theme},
    utils::{
        colors::hsl_to_rgb,
        styles::{HIGHLIGHT_NAME_DARK, HIGHLIGHT_NAME_LIGHT, SYSTEM_CHAT},
    },
};

lazy_static! {
    pub static ref FUZZY_FINDER: SkimMatcherV2 = SkimMatcherV2::default();
}

#[derive(Debug, Clone)]
pub struct MessageData {
    pub time_sent: DateTime<Local>,
    pub author: String,
    pub system: bool,
    pub payload: String,
}

impl MessageData {
    pub fn new(author: String, system: bool, payload: String) -> Self {
        Self {
            time_sent: Local::now(),
            author,
            system,
            payload,
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

    pub fn to_row_and_num_search_results(
        &self,
        frontend_config: &FrontendConfig,
        width: usize,
        search_highlight: Option<String>,
        username_highlight: Option<String>,
        theme_style: Style,
    ) -> (Vec<Spans>, u32) {
        let message = textwrap::fill(
            self.payload.as_str(),
            width - self.author.len() - self.time_sent.to_string().len(),
        );

        let message_spans = message
            .split('\n')
            .map(|s| Span::raw(s.to_owned()))
            .collect::<Vec<Span>>();

        let info = vec![
            Span::from(
                self.time_sent
                    .format(&frontend_config.date_format)
                    .to_string(),
            ),
            Span::from(self.author.clone()),
            message_spans[0].clone(),
        ];

        let extra_message_spans = Spans::from(message_spans[0..].to_vec());

        (vec![Spans::from(info), extra_message_spans], 0)

        // let username_highlight_style = username_highlight.map_or_else(Style::default, |username| {
        //     if Regex::new(format!("^.*{username}.*$").as_str())
        //         .unwrap()
        //         .is_match(&message)
        //     {
        //         match frontend_config.theme {
        //             Theme::Light => HIGHLIGHT_NAME_LIGHT,
        //             _ => HIGHLIGHT_NAME_DARK,
        //         }
        //     } else {
        //         Style::default()
        //     }
        // });

        // let mut num_search_matches = 0;

        // let msg_cells = search_highlight.map_or_else(
        //     || {
        //         // If the user's name appears in a row, highlight it.
        //         message
        //             .split('\n')
        //             .map(|s| {
        //                 Cell::from(Spans::from(vec![Span::styled(
        //                     s.to_owned(),
        //                     username_highlight_style,
        //                 )]))
        //             })
        //             .collect::<Vec<Cell>>()
        //     },
        //     |search| {
        //         // Going through all the rows with a search to see if there's a fuzzy match.
        //         // If there is, highlight said match in red.
        //         message
        //             .split('\n')
        //             .map(|s| {
        //                 let chars = s.chars();

        //                 if let Some((_, indices)) = FUZZY_FINDER.fuzzy_indices(s, search.as_str()) {
        //                     num_search_matches += 1;
        //                     Cell::from(vec![Spans::from(
        //                         chars
        //                             .enumerate()
        //                             .map(|(i, s)| {
        //                                 if indices.contains(&i) {
        //                                     Span::styled(
        //                                         s.to_string(),
        //                                         Style::default()
        //                                             .fg(Color::Red)
        //                                             .add_modifier(Modifier::BOLD),
        //                                     )
        //                                 } else {
        //                                     Span::raw(s.to_string())
        //                                 }
        //                             })
        //                             .collect::<Vec<Span>>(),
        //                     )])
        //                 } else {
        //                     Cell::from(Spans::from(vec![Span::styled(
        //                         s.to_owned(),
        //                         username_highlight_style,
        //                     )]))
        //                 }
        //             })
        //             .collect::<Vec<Cell>>()
        //     },
        // );

        // let mut cell_vector = vec![
        //     Cell::from(self.author).style(if self.system {
        //         SYSTEM_CHAT
        //     } else {
        //         Style::default().fg(self.hash_username(&frontend_config.palette))
        //     }),
        //     msg_cells[0].clone(),
        // ];

        // if frontend_config.date_shown {
        //     cell_vector.insert(
        //         0,
        //         Cell::from(
        //             self.time_sent
        //                 .format(&frontend_config.date_format)
        //                 .to_string(),
        //         ),
        //     );
        // };

        // let mut row_vector = vec![Row::new(cell_vector).style(theme_style)];

        // if msg_cells.len() > 1 {
        //     for cell in msg_cells.iter().skip(1) {
        //         let mut wrapped_msg = vec![Cell::from(""), cell.clone()];

        //         if frontend_config.date_shown {
        //             wrapped_msg.insert(0, Cell::from(""));
        //         }

        //         row_vector.push(Row::new(wrapped_msg));
        //     }
        // }
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
