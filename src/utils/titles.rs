#[allow(dead_code)]
pub struct TitleBar {
    items: Vec<String>,
}

impl TitleBar {
    #[allow(dead_code)]
    pub fn new(formatting: &str) -> Self {
        let items = formatting
            .split(',')
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            })
            .collect();

        Self { items }
    }

    #[allow(dead_code)]
    pub const fn get_items(&self) -> &Vec<String> {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_titlebar_creation_some_items() {
        let titles = TitleBar::new("datetime,channel,filter");

        assert_eq!(
            &vec!["datetime", "channel", "filter"]
                .iter()
                .map(|&s| s.to_string())
                .collect::<Vec<String>>(),
            titles.get_items()
        );
    }

    #[test]
    fn test_titlebar_creation_no_items() {
        let titles = TitleBar::new("");

        let empty_vec: Vec<String> = vec![];

        assert_eq!(&empty_vec, titles.get_items());
    }
}
