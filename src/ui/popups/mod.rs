pub mod help;

use std::collections::VecDeque;

use tui::layout::{Constraint, Direction, Layout, Rect};

const V_WINDOW_PERCENTAGE: u16 = 60;
const H_WINDOW_PERCENTAGE: u16 = 75;

pub enum Centering {
    /// An input box, where the optional u16 determins how far below the input box must be.
    Input(Option<u16>),
    /// A window for showing items, the integer is how many vertical items are to be stored.
    Window(u16),
}

pub fn centered_popup(c: Centering, size: Rect) -> Rect {
    let v_constraint = Constraint::Percentage((100 - V_WINDOW_PERCENTAGE) / 2);
    let h_constraint = Constraint::Percentage((100 - H_WINDOW_PERCENTAGE) / 2);

    match c {
        Centering::Input(w) => {
            let modified_v_constraint = if let Some(i) = w {
                Constraint::Length(size.y - i - 3)
            } else {
                v_constraint
            };

            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([modified_v_constraint, Constraint::Length(3), v_constraint].as_ref())
                .split(size);

            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        h_constraint,
                        Constraint::Percentage(H_WINDOW_PERCENTAGE),
                        h_constraint,
                    ]
                    .as_ref(),
                )
                .split(popup_layout[1])[1]
        }
        Centering::Window(i) => {
            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        v_constraint,
                        // An addition of 4 is made for boarders.
                        Constraint::Length(i + 4),
                        v_constraint,
                    ]
                    .as_ref(),
                )
                .split(size);

            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        h_constraint,
                        Constraint::Percentage(H_WINDOW_PERCENTAGE),
                        h_constraint,
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
