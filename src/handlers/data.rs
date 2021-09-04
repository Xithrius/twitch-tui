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
