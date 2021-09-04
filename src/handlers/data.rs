use textwrap;
use tui::style::{Color, Color::Rgb, Style};
use tui::widgets::{Cell, Row};

#[derive(Debug, Clone)]
pub struct Data {
    pub time_sent: String,
    pub author: String,
    pub message: String,
}

impl Data {
    pub fn new(time_sent: String, author: String, message: String) -> Self {
        Data {
            time_sent,
            author,
            message,
        }
    }

    pub fn hash_username(&self) -> Color {
        let user_bytes = self.author.as_bytes();

        Rgb(user_bytes[0] * 2, user_bytes[1] * 2, user_bytes[2] * 2)
    }

    pub fn to_row(&self) -> Row {
        Row::new(vec![
            Cell::from(self.time_sent.to_string()),
            Cell::from(self.author.to_string()).style(Style::default().fg(self.hash_username())),
            Cell::from(self.message.to_string()),
        ])
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
            ));

            for index in 1..split_message.len() {
                data_vec.push(Data::new(
                    "".to_string(),
                    "".to_string(),
                    split_message[index].to_string(),
                ));
            }
        }

        data_vec
    }
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
        some_data.message = "asdf ".repeat(10);

        let some_vec = some_data.wrap_message(5);
        assert_eq!(some_vec.len(), 10);
    }
}
