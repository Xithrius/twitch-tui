use regex::Regex;

pub struct Filter {
    pub captures: Vec<Regex>,
    pub enabled: bool,
}

#[allow(dead_code)]
impl Filter {
    pub fn new(captures: Vec<String>, enabled: bool) -> Self {
        Filter {
            captures: captures
                .iter()
                .map(|capture| Regex::new(capture).unwrap())
                .collect::<Vec<Regex>>(),
            enabled,
        }
    }

    pub fn contaminated(&self, data: String) -> bool {
        if self.enabled {
            for re in &self.captures {
                if re.is_match(&data) {
                    return true;
                }
            }
        }

        false
    }

    pub fn remove(&mut self, data: &str) {
        if let Some(index) = self.captures.iter().position(|x| x.as_str() == data) {
            self.captures.remove(index);
        }
    }

    pub fn add(&mut self, data: &str) {
        self.captures.push(Regex::new(data).unwrap());
    }
}
