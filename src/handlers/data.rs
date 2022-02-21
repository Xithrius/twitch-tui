use chrono::offset::Local;
use tui::{
    style::{Color, Color::Rgb, Style},
    widgets::{Cell, Row},
};

use crate::{
    handlers::config::{FrontendConfig, Palette},
    utils::{colors::hsl_to_rgb, styles, text::align_text},
};

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
            time_sent: Local::now().format(self.date_format).to_string(),
            author: user,
            system: false,
            payload: PayLoad::Message(message),
        }
    }

    pub fn system(self, message: String) -> Data {
        Data {
            time_sent: Local::now().format(self.date_format).to_string(),
            author: "System".to_string(),
            system: true,
            payload: PayLoad::Message(message),
        }
    }

    pub fn twitch(self, message: String) -> Data {
        Data {
            time_sent: Local::now().format(self.date_format).to_string(),
            author: "Twitch".to_string(),
            system: true,
            payload: PayLoad::Message(message),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Data {
    pub time_sent: String,
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

    pub fn to_row(&self, frontend_config: &FrontendConfig, limit: &usize) -> Vec<Row> {
        if let PayLoad::Message(m) = &self.payload {
            let message = textwrap::fill(m.as_str(), *limit);

            let style = if self.system {
                styles::SYSTEM_CHAT
            } else {
                Style::default().fg(self.hash_username(&frontend_config.palette))
            };

            let mut msg_split = message
                .split('\n')
                .map(|c| c.to_string())
                .collect::<Vec<String>>();

            let mut initial_row_vector = vec![
                Cell::from(align_text(
                    &self.author,
                    frontend_config.username_alignment.as_str(),
                    frontend_config.maximum_username_length,
                ))
                .style(style),
                Cell::from(msg_split[0].to_string()).style(styles::CHAT),
            ];

            if frontend_config.date_shown {
                initial_row_vector.insert(0, Cell::from(self.time_sent.to_string()));
            }

            let mut row_vector: Vec<Row> = vec![Row::new(initial_row_vector)];

            if msg_split.len() > 1 {
                for msg in msg_split.drain(1..) {
                    let mut wrapped_msg = vec![Cell::from(""), Cell::from(msg).style(styles::CHAT)];

                    if frontend_config.date_shown {
                        wrapped_msg.insert(0, Cell::from(""));
                    }

                    row_vector.push(Row::new(wrapped_msg));
                }
            }

            row_vector
        } else {
            panic!("Data.to_row() can only take message payloads.")
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Local;
    use tui::style::Color::Rgb;

    use super::*;

    fn create_data() -> Data {
        Data {
            time_sent: Local::now().format("%c").to_string(),
            author: "human".to_string(),
            system: false,
            payload: PayLoad::Message("beep boop".to_string()),
        }
    }

    #[test]
    fn test_username_hash() {
        assert_eq!(
            create_data().hash_username(&Palette::Pastel),
            Rgb(159, 223, 221)
        );
    }
}
