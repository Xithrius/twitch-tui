use std::iter;

use crate::handlers::config::FrontendConfig;

pub const PRIVATE_USE_UNICODE: char = '\u{10EEEE}';
pub const ZERO_WIDTH_SPACE: char = '\u{200B}';
pub const ZERO_WIDTH_SPACE_STR: &str = "\u{200B}";

pub const fn get_emote_offset(width: u16, cell_width: u16, cols: u16) -> (u16, u16) {
    let w = (width + if cols % 2 == 0 { 0 } else { cell_width }).div_ceil(2);

    let (pxo, co) = (w % cell_width, w / cell_width);

    let (pxo, co) = if pxo == 0 {
        (0, co)
    } else {
        (cell_width - pxo, co + 1)
    };

    (pxo, co)
}

/// Unicode placeholders use [`PRIVATE_USE_UNICODE`] characters as placeholders for images.
///
/// A unicode placeholder consists of multiple [`PRIVATE_USE_UNICODE`] so that it takes the same amount of space on screen as the image.
///
/// The format for a Unicode placeholder is `{PRIVATE_USE_UNICODE} * width`
///
/// [Reference](https://sw.kovidgoyal.net/kitty/graphics-protocol/#unicode-placeholders)
pub struct UnicodePlaceholder(usize);

impl UnicodePlaceholder {
    pub const fn new(width: usize) -> Self {
        assert!(width > 0);
        Self(width)
    }

    #[allow(unused)]
    pub const fn len(&self) -> usize {
        PRIVATE_USE_UNICODE.len_utf8() * self.0
    }

    pub fn iter(&'_ self) -> impl Iterator<Item = char> + '_ {
        let mut count = 0;
        iter::from_fn(move || {
            count += 1;

            if count > self.0 {
                None
            } else {
                Some(PRIVATE_USE_UNICODE)
            }
        })
    }

    #[allow(unused)]
    pub fn string(&self) -> String {
        let mut s = String::with_capacity(self.len());

        s.extend(self.iter());

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emote_offset_1_col() {
        // 1 col, even cell width.
        assert_eq!(get_emote_offset(2, 10, 1), (4, 1));
        assert_eq!(get_emote_offset(8, 10, 1), (1, 1));
        assert_eq!(get_emote_offset(9, 10, 1), (0, 1));
        assert_eq!(get_emote_offset(10, 10, 1), (0, 1));

        // 1 col, odd cell width.
        assert_eq!(get_emote_offset(2, 13, 1), (5, 1));
        assert_eq!(get_emote_offset(8, 13, 1), (2, 1));
        assert_eq!(get_emote_offset(12, 13, 1), (0, 1));
        assert_eq!(get_emote_offset(13, 13, 1), (0, 1));
    }

    #[test]
    fn emote_offset_2_cols() {
        // 2 cols, even cell width.
        assert_eq!(get_emote_offset(2, 10, 2), (9, 1));
        assert_eq!(get_emote_offset(8, 10, 2), (6, 1));
        assert_eq!(get_emote_offset(9, 10, 2), (5, 1));
        assert_eq!(get_emote_offset(10, 10, 2), (5, 1));
        assert_eq!(get_emote_offset(11, 10, 2), (4, 1));
        assert_eq!(get_emote_offset(12, 10, 2), (4, 1));
        assert_eq!(get_emote_offset(20, 10, 2), (0, 1));

        // 2 cols, odd cell width.
        assert_eq!(get_emote_offset(2, 13, 2), (12, 1));
        assert_eq!(get_emote_offset(8, 13, 2), (9, 1));
        assert_eq!(get_emote_offset(12, 13, 2), (7, 1));
        assert_eq!(get_emote_offset(13, 13, 2), (6, 1));
        assert_eq!(get_emote_offset(14, 13, 2), (6, 1));
        assert_eq!(get_emote_offset(26, 13, 2), (0, 1));
    }

    #[test]
    fn emote_offset_3_cols() {
        // 3 cols, even cell width.
        assert_eq!(get_emote_offset(2, 10, 3), (4, 1));
        assert_eq!(get_emote_offset(9, 10, 3), (0, 1));
        assert_eq!(get_emote_offset(10, 10, 3), (0, 1));
        assert_eq!(get_emote_offset(11, 10, 3), (9, 2));
        assert_eq!(get_emote_offset(12, 10, 3), (9, 2));
        assert_eq!(get_emote_offset(14, 10, 3), (8, 2));
        assert_eq!(get_emote_offset(20, 10, 3), (5, 2));
        assert_eq!(get_emote_offset(30, 10, 3), (0, 2));

        // 3 cols, odd cell width.
        assert_eq!(get_emote_offset(2, 13, 3), (5, 1));
        assert_eq!(get_emote_offset(12, 13, 3), (0, 1));
        assert_eq!(get_emote_offset(13, 13, 3), (0, 1));
        assert_eq!(get_emote_offset(14, 13, 3), (12, 2));
        assert_eq!(get_emote_offset(15, 13, 3), (12, 2));
        assert_eq!(get_emote_offset(26, 13, 3), (6, 2));
        assert_eq!(get_emote_offset(29, 13, 3), (5, 2));
        assert_eq!(get_emote_offset(39, 13, 3), (0, 2));
    }

    #[test]
    fn emote_offset_4_cols() {
        // 4 cols, even cell width.
        assert_eq!(get_emote_offset(2, 10, 4), (9, 1));
        assert_eq!(get_emote_offset(8, 10, 4), (6, 1));
        assert_eq!(get_emote_offset(9, 10, 4), (5, 1));
        assert_eq!(get_emote_offset(10, 10, 4), (5, 1));
        assert_eq!(get_emote_offset(11, 10, 4), (4, 1));
        assert_eq!(get_emote_offset(12, 10, 4), (4, 1));
        assert_eq!(get_emote_offset(20, 10, 4), (0, 1));
        assert_eq!(get_emote_offset(25, 10, 4), (7, 2));
        assert_eq!(get_emote_offset(30, 10, 4), (5, 2));
        assert_eq!(get_emote_offset(40, 10, 4), (0, 2));

        // 4 cols, odd cell width.
        assert_eq!(get_emote_offset(2, 13, 4), (12, 1));
        assert_eq!(get_emote_offset(8, 13, 4), (9, 1));
        assert_eq!(get_emote_offset(12, 13, 4), (7, 1));
        assert_eq!(get_emote_offset(13, 13, 4), (6, 1));
        assert_eq!(get_emote_offset(14, 13, 4), (6, 1));
        assert_eq!(get_emote_offset(26, 13, 4), (0, 1));
        assert_eq!(get_emote_offset(31, 13, 4), (10, 2));
        assert_eq!(get_emote_offset(34, 13, 4), (9, 2));
        assert_eq!(get_emote_offset(52, 13, 4), (0, 2));
    }

    #[test]
    fn unicode_placeholders() {
        assert_eq!(
            UnicodePlaceholder::new(1).string(),
            format!("{PRIVATE_USE_UNICODE}")
        );
        assert_eq!(
            UnicodePlaceholder::new(2).string(),
            format!("{PRIVATE_USE_UNICODE}{PRIVATE_USE_UNICODE}")
        );
        assert_eq!(
            UnicodePlaceholder::new(3).string(),
            format!("{PRIVATE_USE_UNICODE}{PRIVATE_USE_UNICODE}{PRIVATE_USE_UNICODE}")
        );

        let up = UnicodePlaceholder::new(3);

        assert_eq!(up.len(), PRIVATE_USE_UNICODE.len_utf8() * 3);

        let mut iter = up.iter();

        assert_eq!(iter.next(), Some(PRIVATE_USE_UNICODE));
        assert_eq!(iter.next(), Some(PRIVATE_USE_UNICODE));
        assert_eq!(iter.next(), Some(PRIVATE_USE_UNICODE));
        assert_eq!(iter.next(), None);
    }
}
