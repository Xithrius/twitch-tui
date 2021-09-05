use crate::handlers::config::Palette;
use tui::style::{Color, Color::Rgb, Style};
use tui::widgets::{Cell, Row};

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

    pub fn hash_username(&self, palette: &Palette) -> Color {
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

    pub fn to_row(&self, palette: &Palette) -> Row {
        return if self.empty {
            Row::new(vec![
                Cell::from("".to_string()),
                Cell::from("".to_string()),
                Cell::from(self.message.to_string()),
            ])
        } else {
            Row::new(vec![
                Cell::from(self.time_sent.to_string()),
                Cell::from(self.author.to_string())
                    .style(Style::default().fg(self.hash_username(palette))),
                Cell::from(self.message.to_string()),
            ])
        };
    }

    pub fn wrap_message(self, limit: usize) -> Vec<Data> {
        let mut data_vec = Vec::new();

        let split_message = textwrap::fill(self.message.as_str(), limit)
            .split("\n")
            .map(|m| m.to_string())
            .collect::<Vec<String>>();

        if split_message.len() == 1 {
            data_vec.push(self);
        } else if split_message.len() > 1 {
            data_vec.push(Data::new(
                self.time_sent,
                self.author,
                split_message[0].to_string(),
                false,
            ));

            for index in 1..split_message.len() {
                data_vec.push(Data::new(
                    "".to_string(),
                    "".to_string(),
                    split_message[index].to_string(),
                    true,
                ));
            }
        }

        data_vec
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
            create_data().hash_username(),
            Rgb(104 * 2, 117 * 2, 109 * 2)
        );
    }

    #[test]
    fn test_data_message_wrapping() {
        let mut some_data = create_data();
        some_data.message = "asdf ".repeat(39);
        assert_eq!(some_data.message.len(), 195);

        let some_vec = some_data.wrap_message(157);
        assert_eq!(some_vec.len(), 2);
    }
}
