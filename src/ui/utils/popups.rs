pub fn centered_popup(size: Rect, terminal_height: u16, h_constraints: &[Constraint]) -> Rect {
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
        .constraints(h_constraints.as_ref())
        .split(popup_layout[1])[1]
}
