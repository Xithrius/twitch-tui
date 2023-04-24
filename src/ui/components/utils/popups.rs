use tui::layout::{Constraint, Direction, Layout, Rect};

const HORIZONTAL_CONSTRAINTS: [Constraint; 3] = [
    Constraint::Percentage(15),
    Constraint::Percentage(70),
    Constraint::Percentage(15),
];

// pub fn centered_rect(size: Rect, terminal_height: u16) -> Rect {
//     let popup_layout = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints(
//             [
//                 Constraint::Length((terminal_height / 2) - 6),
//                 Constraint::Length(3),
//                 Constraint::Min(0),
//             ]
//             .as_ref(),
//         )
//         .split(size);

//     Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints(HORIZONTAL_CONSTRAINTS.as_ref())
//         .split(popup_layout[1])[1]
// }

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                // Constraint::Percentage(percent_y),
                Constraint::Length(3),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

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
