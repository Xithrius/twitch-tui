/// <https://css-tricks.com/converting-color-spaces-in-javascript/#hsl-to-rgb/>
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_white_to_rgb() {
        let white_rgb = hsl_to_rgb(0.0, 0.0, 0.0);

        assert_eq!([0, 0, 0], white_rgb);
    }

    #[test]
    fn test_black_to_rgb() {
        let black_rgb = hsl_to_rgb(255.0, 255.0, 255.0);

        assert_eq!([255, 255, 0], black_rgb);
    }
}
