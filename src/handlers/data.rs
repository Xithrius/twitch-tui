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
        text::align_text,
    },
};

lazy_static! {
    pub static ref FUZZY_FINDER: SkimMatcherV2 = SkimMatcherV2::default();
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum PayLoad {
    Message(String),
    Err(String),
}

#[derive(Debug, Copy, Clone)]
pub struct DataBuilder<'conf> {
    pub date_format: &'conf str,
}

impl<'conf> DataBuilder<'conf> {
    pub const fn new(date_format: &'conf str) -> Self {
        DataBuilder { date_format }
    }

    pub fn user(user: String, message: String) -> Data {
        Data {
            time_sent: Local::now(),
            author: user,
            system: false,
            payload: PayLoad::Message(message),
        }
    }

    pub fn system(self, message: String) -> Data {
        Data {
            time_sent: Local::now(),
            author: "System".to_string(),
            system: true,
            payload: PayLoad::Message(message),
        }
    }

    pub fn twitch(self, message: String) -> Data {
        Data {
            time_sent: Local::now(),
            author: "Twitch".to_string(),
            system: true,
            payload: PayLoad::Message(message),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Data {
    pub time_sent: DateTime<Local>,
    pub author: String,
    pub system: bool,
    pub payload: PayLoad,
}

impl Data {
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
        limit: usize,
        search_highlight: Option<String>,
        username_highlight: Option<String>,
        theme_style: Style,
    ) -> (Vec<Row>, u32) {
        let message = if let PayLoad::Message(m) = &self.payload {
            textwrap::fill(m.as_str(), limit)
        } else {
            panic!("Data.to_row() can only take an enum of PayLoad::Message.");
        };

        let username_highlight_style = username_highlight.map_or_else(Style::default, |username| {
            if Regex::new(format!("^.*{}.*$", username).as_str())
                .unwrap()
                .is_match(&message)
            {
                match frontend_config.theme {
                    Theme::Light => HIGHLIGHT_NAME_LIGHT,
                    _ => HIGHLIGHT_NAME_DARK,
                }
            } else {
                Style::default()
            }
        });

        let mut num_search_matches = 0;
        let msg_cells = search_highlight.map_or_else(
            || {
                message
                    .split('\n')
                    .map(|s| {
                        Cell::from(Spans::from(vec![Span::styled(
                            s.to_owned(),
                            username_highlight_style,
                        )]))
                    })
                    .collect::<Vec<Cell>>()
            },
            |search| {
                message
                    .split('\n')
                    .map(|s| {
                        let chars = s.chars();

                        if let Some((_, indices)) = FUZZY_FINDER.fuzzy_indices(s, search.as_str()) {
                            num_search_matches += 1;
                            Cell::from(vec![Spans::from(
                                chars
                                    .enumerate()
                                    .map(|(i, s)| {
                                        if indices.contains(&i) {
                                            Span::styled(
                                                s.to_string(),
                                                Style::default()
                                                    .fg(Color::Red)
                                                    .add_modifier(Modifier::BOLD),
                                            )
                                        } else {
                                            Span::raw(s.to_string())
                                        }
                                    })
                                    .collect::<Vec<Span>>(),
                            )])
                        } else {
                            Cell::from(Spans::from(vec![Span::styled(
                                s.to_owned(),
                                username_highlight_style,
                            )]))
                        }
                    })
                    .collect::<Vec<Cell>>()
            },
        );

        let mut cell_vector = vec![
            Cell::from(align_text(
                &self.author,
                frontend_config.username_alignment,
                frontend_config.maximum_username_length,
            ))
            .style(if self.system {
                SYSTEM_CHAT
            } else {
                Style::default().fg(self.hash_username(&frontend_config.palette))
            }),
            msg_cells[0].clone(),
        ];

        if frontend_config.date_shown {
            cell_vector.insert(
                0,
                Cell::from(
                    self.time_sent
                        .format(&frontend_config.date_format)
                        .to_string(),
                ),
            );
        };

        let mut row_vector = vec![Row::new(cell_vector).style(theme_style)];

        if msg_cells.len() > 1 {
            for cell in msg_cells.iter().skip(1) {
                let mut wrapped_msg = vec![Cell::from(""), cell.clone()];

                if frontend_config.date_shown {
                    wrapped_msg.insert(0, Cell::from(""));
                }

                row_vector.push(Row::new(wrapped_msg));
            }
        }

        (row_vector, num_search_matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_hash() {
        assert_eq!(
            Data {
                time_sent: Local::now(),
                author: "human".to_string(),
                system: false,
                payload: PayLoad::Message("beep boop".to_string()),
            }
            .hash_username(&Palette::Pastel),
            Rgb(159, 223, 221)
        );
    }
}
