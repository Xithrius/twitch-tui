use tui::layout::{Constraint, Direction, Layout, Rect};

pub mod channels;
pub mod help;

const HORIZONTAL_CONSTRAINTS: [Constraint; 3] = [
    Constraint::Percentage(15),
    Constraint::Percentage(70),
    Constraint::Percentage(15),
];

#[allow(dead_code)]
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
            let s = i + 2;

            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(match v {
                        Centering::Height(terminal_height) => (terminal_height / 2) as u16 - 3,
                        Centering::Middle(terminal_height) => {
                            ((terminal_height / 2)
                                - (if terminal_height > (s / 2) { s / 2 } else { 0 }))
                                as u16
                        }
                    }),
                    Constraint::Min(i),
                    match v {
                        Centering::Height(_) => Constraint::Min(0),
                        Centering::Middle(terminal_height) => Constraint::Length(
                            ((terminal_height / 2)
                                - (if terminal_height > (s / 2) { s / 2 } else { 0 }))
                                as u16,
                        ),
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
