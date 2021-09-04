use textwrap;
use tui::style::{Color::Rgb, Modifier, Style};
use tui::widgets::{Cell, Row};

#[derive(Debug, Clone)]
pub struct Data {
    pub time_sent: String,
    pub author: String,
    pub message: String,
}

impl Data {
    pub fn to_row(&self) -> Row {
        let user_bytes = self.author.as_bytes();
        let user_color = Rgb(user_bytes[0] * 2, user_bytes[1] * 2, user_bytes[2] * 2);

        Row::new(vec![
            Cell::from(self.time_sent.to_string()),
            Cell::from(self.author.to_string()).style(Style::default().fg(user_color)),
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
            data_vec.push(Data {
                time_sent: self.time_sent,
                author: self.author,
                message: split_message[0].to_string(),
            });

            for index in 1..split_message.len() {
                data_vec.push(Data {
                    time_sent: "".to_string(),
                    author: "".to_string(),
                    message: split_message[index].clone(),
                });
            }
        }

        data_vec
    }
}
