pub mod help;

use std::collections::VecDeque;

use tui::layout::{Constraint, Direction, Layout, Rect};

pub enum Centering {
    Input(u16, u16),
    Window(u16, u16),
}

pub fn centered_popup(c: Centering, size: Rect) -> Rect {
    match c {
        Centering::Window(percent_x, percent_y) => {
            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage((100 - percent_y) / 2),
                        Constraint::Percentage(percent_y),
                        Constraint::Percentage((100 - percent_y) / 2),
                    ]
                    .as_ref(),
                )
                .split(size);

            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage((100 - percent_x) / 2),
                        Constraint::Percentage(percent_x),
                        Constraint::Percentage((100 - percent_x) / 2),
                    ]
                    .as_ref(),
                )
                .split(popup_layout[1])[1]
        }
        Centering::Input(percent_x, percent_y) => {
            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage((100 - percent_y) / 2),
                        Constraint::Length(3),
                        Constraint::Percentage((100 - percent_y) / 2),
                    ]
                    .as_ref(),
                )
                .split(size);

            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage((100 - percent_x) / 2),
                        Constraint::Percentage(percent_x),
                        Constraint::Percentage((100 - percent_x) / 2),
                    ]
                    .as_ref(),
                )
                .split(popup_layout[1])[1]
        }
    }
}

#[allow(dead_code)]
pub fn scroll_view<T: std::marker::Copy>(
    v: VecDeque<T>,
    offset: usize,
    amount: usize,
) -> VecDeque<T> {
    // If the offset is at 0 or at the bottom of the input, then there's no need to move.
    if offset == 0 && amount == v.len() {
        return v;
    }

    if amount == 0 {
        v.range(offset..).copied().collect::<VecDeque<T>>()
    } else if offset + amount >= v.len() {
        v.range(v.len() - amount..)
            .copied()
            .collect::<VecDeque<T>>()
    } else {
        v.range(offset..offset + amount)
            .copied()
            .collect::<VecDeque<T>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> VecDeque<i32> {
        VecDeque::from([1, 2, 3, 4, 5])
    }

    #[test]
    fn test_offset_1_all_elements() {
        assert_eq!(scroll_view(setup(), 1, 0), VecDeque::from([2, 3, 4, 5]));
    }

    #[test]
    fn test_offset_1_some_elements() {
        assert_eq!(scroll_view(setup(), 2, 3), VecDeque::from([3, 4, 5]));
    }

    #[test]
    fn test_offset_plus_amount_over_length() {
        assert_eq!(scroll_view(setup(), 10, 3), VecDeque::from([3, 4, 5]));
    }

    #[test]
    fn test_offset_and_amount_centered() {
        assert_eq!(scroll_view(setup(), 1, 3), VecDeque::from([2, 3, 4]));
    }
}
