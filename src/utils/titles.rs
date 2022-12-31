pub struct TitleBar {
    items: Vec<String>,
}

impl TitleBar {
    pub fn new(formatting: String) -> Self {
        Self {
            items: formatting.split(',').map(String::from).collect(),
        }
    }

    pub const fn get_items(&self) -> Vec<String> {
        self.items.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation_of_items() {
        let test = TitleBar::new("datetime,channel,filter".to_string());

        assert_eq!(
            vec!["datetime", "channel", "filter"]
                .iter()
                .map(|&s| s.to_string())
                .collect::<Vec<String>>(),
            test.get_items()
        );
    }
}
