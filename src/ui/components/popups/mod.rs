use tui::layout::{Constraint, Direction, Layout, Rect};

pub mod channels;

const HORIZONTAL_CONSTRAINTS: [Constraint; 3] = [
    Constraint::Percentage(15),
    Constraint::Percentage(70),
    Constraint::Percentage(15),
];

pub fn centered_popup(size: Rect, terminal_height: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length((terminal_height / 2) - 6),
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
