use tui::style::{Color, Color::Rgb, Style};
use tui::widgets::{Cell, Row};

use crate::handlers::config::Palette;

#[derive(Debug, Clone)]
pub struct Data {
    pub time_sent: String,
    pub author: String,
    pub message: String,
    pub empty: bool,
}

impl Data {
    pub fn new(time_sent: String, author: String, message: String, empty: bool) -> Self {
        Data {
            time_sent,
            author,
            message,
            empty,
        }
    }

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

    pub fn to_row(&self, palette: &Palette, limit: usize) -> (u16, Row) {
        let message = textwrap::fill(self.message.as_str(), limit);

        let mut row = Row::new(vec![
            Cell::from(self.time_sent.to_string()),
            Cell::from(self.author.to_string())
                .style(Style::default().fg(self.hash_username(palette))),
            Cell::from(message.to_string()),
        ]);

        let msg_height = message.split("\n").collect::<Vec<&str>>().len() as u16;

        if msg_height > 1 {
            row = row.height(msg_height);
        }

        (msg_height, row)
    }
}

// https://css-tricks.com/converting-color-spaces-in-javascript/#hsl-to-rgb
fn hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> [u8; 3] {
    // Color intensity
    let chroma = (1. - (2. * lightness - 1.).abs()) * saturation;

    // Second largest component
    let x = chroma * (1. - ((hue / 60.) % 2. - 1.).abs());

    // Amount to match lightness
    let m = lightness - chroma / 2.;

    // Convert to rgb based on color wheel section
    let (mut red, mut green, mut blue) = match hue.round() as i32 {
        0..=60 => (chroma, x, 0.),
        61..=120 => (x, chroma, 0.),
        121..=180 => (0., chroma, x),
        181..=240 => (0., x, chroma),
        241..=300 => (x, 0., chroma),
        301..=360 => (chroma, 0., x),
        _ => {
            panic!("Invalid hue!");
        }
    };

    // Add amount to each channel to match lightness
    red = (red + m) * 255.;
    green = (green + m) * 255.;
    blue = (blue + m) * 255.;

    [red as u8, green as u8, blue as u8]
}

#[cfg(test)]
mod tests {
    use chrono::Local;
    use tui::style::Color::Rgb;

    use super::*;

    fn create_data() -> Data {
        Data::new(
            Local::now().format("%c").to_string(),
            "human".to_string(),
            "beep boop".to_string(),
            false,
        )
    }

    #[test]
    fn test_username_hash() {
        assert_eq!(
            create_data().hash_username(&Palette::Pastel),
            Rgb(159, 223, 221)
        );
    }

    // #[test]
    // fn test_data_message_wrapping() {
    //     let mut some_data = create_data();
    //     some_data.message = "asdf ".repeat(39);
    //     assert_eq!(some_data.message.len(), 195);
    //
    //     let some_vec = some_data.wrap_message(157);
    //     assert_eq!(some_vec.len(), 2);
    // }
}
