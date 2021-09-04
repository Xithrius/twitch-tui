use textwrap;

#[derive(Debug, Clone)]
pub struct Data {
    pub time_sent: String,
    pub author: String,
    pub message: String,
}

impl Data {
    pub fn to_vec(&self) -> Vec<String> {
        return vec![
            self.time_sent.to_string(),
            self.author.to_string(),
            self.message.to_string(),
        ];
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

#[cfg(test)]
mod tests {
    use chrono::Local;

    use super::*;

    #[test]
    fn test_data() {
        let some_time = Local::now().format("%c");

        let var = Data {
            time_sent: some_time.to_string(),
            author: "A human".to_string(),
            message: "beep boop".to_string(),
        };

        let var_vector_test = vec![
            some_time.to_string(),
            "A human".to_string(),
            "beep boop".to_string(),
        ];

        assert_eq!(var.to_vec(), var_vector_test);
    }
}
