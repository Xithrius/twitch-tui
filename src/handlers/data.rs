use chrono::{offset::Local, DateTime};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use lazy_static::lazy_static;
use tui::{
    style::{Color, Color::Rgb, Modifier, Style},
    text::{Span, Spans},
    widgets::{Cell, Row},
};

use crate::{
    handlers::config::{FrontendConfig, Palette},
    utils::{colors::hsl_to_rgb, styles, text::align_text},
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
    pub fn new(date_format: &'conf str) -> Self {
        DataBuilder { date_format }
    }

    pub fn user(self, user: String, message: String) -> Data {
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
        let hash = self
            .author
            .as_bytes()
            .iter()
            .map(|&b| b as u32)
            .sum::<u32>() as f64;

        let (hue, saturation, lightness) = match palette {
            Palette::Pastel => (hash % 360. + 1., 0.5, 0.75),
            Palette::Vibrant => (hash % 360. + 1., 1., 0.6),
            Palette::Warm => ((hash % 100. + 1.) * 1.2, 0.8, 0.7),
            Palette::Cool => ((hash % 100. + 1.) * 1.2 + 180., 0.6, 0.7),
        };

        let rgb = hsl_to_rgb(hue, saturation, lightness);

        Rgb(rgb[0], rgb[1], rgb[2])
    }

    pub fn to_row(
        &self,
        frontend_config: &FrontendConfig,
        limit: &usize,
        highlight: Option<String>,
    ) -> Vec<Row> {
        let message = if let PayLoad::Message(m) = &self.payload {
            textwrap::fill(m.as_str(), *limit)
        } else {
            panic!("Data.to_row() can only take an enum of PayLoad::Message.");
        };

        let msg_cells: Vec<Cell> = if let Some(search) = highlight {
            message
                .split('\n')
                .map(|s| {
                    let chars = s.chars();

                    if let Some((_, indices)) = FUZZY_FINDER.fuzzy_indices(s, search.as_str()) {
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
                        Cell::from(s.to_owned())
                    }
                })
                .collect::<Vec<Cell>>()
        } else {
            message
                .split('\n')
                .map(|c| Cell::from(c.to_string()))
                .collect::<Vec<Cell>>()
        };

        let mut cell_vector = vec![
            Cell::from(align_text(
                &self.author,
                frontend_config.username_alignment.as_str(),
                frontend_config.maximum_username_length,
            ))
            .style(if self.system {
                styles::SYSTEM_CHAT
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

        let mut row_vector = vec![Row::new(cell_vector)];

        if msg_cells.len() > 1 {
            for cell in msg_cells.iter().skip(1) {
                let mut wrapped_msg = vec![Cell::from(""), cell.to_owned().style(styles::CHAT)];

                if frontend_config.date_shown {
                    wrapped_msg.insert(0, Cell::from(""));
                }

                row_vector.push(Row::new(wrapped_msg));
            }
        }

        row_vector
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
