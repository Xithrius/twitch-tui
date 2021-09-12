use tui::style::{Color, Modifier, Style};

pub enum WindowStyles {
    BoarderName,
    ColumnTitle,
    Chat,
    SystemChat,
}

impl WindowStyles {
    pub fn new(style_type: WindowStyles) -> Style {
        let style = Style::default();

        match style_type {
            WindowStyles::BoarderName => style.fg(Color::White),
            WindowStyles::ColumnTitle => style.fg(Color::LightCyan).add_modifier(Modifier::BOLD),
            WindowStyles::Chat => style.fg(Color::White),
            WindowStyles::SystemChat => style.fg(Color::Red).add_modifier(Modifier::BOLD),
        }
    }
}

// https://css-tricks.com/converting-color-spaces-in-javascript/#hsl-to-rgb
pub fn hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> [u8; 3] {
    // Color intensity
    let chroma = (1. - (2. * lightness - 1.).abs()) * saturation;

    // Second largest component
    let x = chroma * (1. - ((hue / 60.) % 2. - 1.).abs());

    // Amount to match lightness
    let m = lightness - chroma / 2.;

    // Convert to rgb based on color wheel section
    let (mut red, mut green, mut blue) = match hue.round() as i32 {
        0..=60 => (chroma, x, 0.),
        61..=120 => (x, chroma, 0.),
        121..=180 => (0., chroma, x),
        181..=240 => (0., x, chroma),
        241..=300 => (x, 0., chroma),
        301..=360 => (chroma, 0., x),
        _ => {
            panic!("Invalid hue!");
        }
    };

    // Add amount to each channel to match lightness
    red = (red + m) * 255.;
    green = (green + m) * 255.;
    blue = (blue + m) * 255.;

    [red as u8, green as u8, blue as u8]
}
