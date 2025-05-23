pub struct Scrolling {
    /// Offset of scroll
    offset: usize,
    /// If the scrolling is currently inverted
    inverted: bool,
}

impl Scrolling {
    pub const fn new(inverted: bool) -> Self {
        Self {
            offset: 0,
            inverted,
        }
    }

    pub const fn is_inverted(&self) -> bool {
        self.inverted
    }

    pub const fn up(&mut self) {
        self.offset = self.offset.saturating_add(1);
    }

    pub const fn down(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }

    pub const fn jump_to(&mut self, index: usize) {
        self.offset = index;
    }

    pub const fn get_offset(&self) -> usize {
        self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_scroll_overflow_not_inverted() {
        let mut scroll = Scrolling::new(false);
        assert_eq!(scroll.get_offset(), 0);

        scroll.down();
        assert_eq!(scroll.get_offset(), 0);
    }
}
