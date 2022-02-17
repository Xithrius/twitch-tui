pub mod channels;
pub mod help;
pub mod messages;

use std::collections::VecDeque;

use tui::layout::{Constraint, Direction, Layout, Rect};

const HORIZONTAL_CONSTRAINTS: [Constraint; 3] = [
    Constraint::Percentage(15),
    Constraint::Percentage(70),
    Constraint::Percentage(15),
];

pub enum Centering {
    Height(u16),
    Middle(u16),
}

pub enum WindowType {
    /// An input window, with the integer representing the height of the terminal
    Input(u16),
    /// A window containing either some specified terminal height, or in the middle,
    /// with an integer describing the amount of vertically stored items
    Window(Centering, u16),
}

pub fn centered_popup(c: WindowType, size: Rect) -> Rect {
    match c {
        WindowType::Input(v) => {
            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length((v / 2) as u16 - 6),
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ]
                    .as_ref(),
                )
                .split(size);

            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(HORIZONTAL_CONSTRAINTS.as_ref())
                .split(popup_layout[1])[1]
        }
        WindowType::Window(v, i) => {
            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(match v {
                        Centering::Height(terminal_height) => (terminal_height / 2) as u16 - 3,
                        Centering::Middle(terminal_height) => ((terminal_height - i) / 2) as u16,
                    }),
                    Constraint::Length(i),
                    match v {
                        Centering::Height(_) => Constraint::Min(0),
                        Centering::Middle(terminal_height) => {
                            Constraint::Length(((terminal_height - i) / 2) as u16 - 3)
                        }
                    },
                ])
                .split(size);

            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(HORIZONTAL_CONSTRAINTS.as_ref())
                .split(popup_layout[1])[1]
        }
    }
}

pub fn scroll_view<T: std::marker::Copy>(
    v: VecDeque<T>,
    offset: usize,
    amount: usize,
) -> VecDeque<T> {
    if offset > v.len() {
        panic!(
            "Scroll offset is {}, but length of VecDeque is {}.",
            offset,
            v.len()
        );
    }
    // If the offset is at 0 or at the bottom of the input, then there's no need to move.
    else if (offset == 0 && amount == v.len()) || v.is_empty() {
        v
    }
    // If there's no amount specified, return everything behind and including the offset index values,
    // or when the offset and the amount wanted goes over the length of the VecDeque.
    else if amount == 0 || offset + amount > v.len() {
        v.range(offset..).copied().collect::<VecDeque<T>>()
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
    #[should_panic(expected = "Scroll offset is 10, but length of VecDeque is 5.")]
    fn test_offset_plus_amount_over_length() {
        scroll_view(setup(), 10, 3);
    }

    #[test]
    #[should_panic(expected = "Scroll offset is 3, but length of VecDeque is 0.")]
    fn test_zero_length_input_some_offset() {
        scroll_view(vec![].iter().copied().collect::<VecDeque<i32>>(), 3, 0);
    }

    #[test]
    fn test_zero_length_input_no_offset() {
        let empty_deq: VecDeque<i32> = scroll_view(VecDeque::from([]), 0, 3);

        assert_eq!(empty_deq, VecDeque::from([]));
    }

    #[test]
    fn test_no_offset_no_amount() {
        let empty_deq: VecDeque<i32> = scroll_view(VecDeque::from([]), 0, 0);

        assert_eq!(empty_deq, VecDeque::from([]));
    }

    #[test]
    fn test_offset_1_all_elements() {
        assert_eq!(scroll_view(setup(), 1, 0), VecDeque::from([2, 3, 4, 5]));
    }

    #[test]
    fn test_no_offset_some_amount() {
        assert_eq!(scroll_view(setup(), 0, 3), VecDeque::from([1, 2, 3]));
    }

    #[test]
    fn test_some_offset_some_amount() {
        assert_eq!(scroll_view(setup(), 2, 2), VecDeque::from([3, 4]));
    }

    #[test]
    fn test_offset_and_amount_centered() {
        assert_eq!(scroll_view(setup(), 1, 3), VecDeque::from([2, 3, 4]));
    }
}
