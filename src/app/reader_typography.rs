//! Reader-specific body typography.
//!
//! Reader pages deliberately use an independent font preference boundary so
//! global UI typography remains stable. Inter and Atkinson reuse the existing
//! UI strikes; Serif uses generated printable-ASCII DejaVu Serif bitmap
//! strikes. TXT normalization converts unsupported punctuation before layout.

use embedded_graphics::pixelcolor::BinaryColor;

use super::{
    display::{UiFontFamily, UiFontSize},
    reader_serif_assets::{SERIF_LARGE, SERIF_MEDIUM, SERIF_SMALL, SERIF_XLARGE},
    typography::{style_for, UiTextRole, UiTextStyle},
};
use crate::reader::{BookFont, BookFontSize, ReadingTheme};

/// Resolve one Reader body strike without affecting global UI preferences.
#[must_use]
pub const fn reader_body_style(
    family: BookFont,
    size: BookFontSize,
    _theme: ReadingTheme,
) -> UiTextStyle {
    match family {
        BookFont::Inter => style_for(
            UiFontFamily::Inter,
            ui_profile(size),
            ui_role(size),
            BinaryColor::On,
        ),
        BookFont::AtkinsonHyperlegible => style_for(
            UiFontFamily::AtkinsonHyperlegible,
            ui_profile(size),
            ui_role(size),
            BinaryColor::On,
        ),
        BookFont::Serif => UiTextStyle::new(serif_font(size), BinaryColor::On),
    }
}

#[must_use]
const fn ui_profile(size: BookFontSize) -> UiFontSize {
    match size {
        BookFontSize::Small => UiFontSize::Compact,
        BookFontSize::Medium => UiFontSize::Standard,
        BookFontSize::Large | BookFontSize::XLarge => UiFontSize::Large,
    }
}

#[must_use]
const fn ui_role(size: BookFontSize) -> UiTextRole {
    match size {
        BookFontSize::Small | BookFontSize::Medium | BookFontSize::Large => UiTextRole::Body,
        BookFontSize::XLarge => UiTextRole::Heading,
    }
}

#[must_use]
const fn serif_font(size: BookFontSize) -> &'static super::typography::BitmapFont {
    match size {
        BookFontSize::Small => &SERIF_SMALL,
        BookFontSize::Medium => &SERIF_MEDIUM,
        BookFontSize::Large => &SERIF_LARGE,
        BookFontSize::XLarge => &SERIF_XLARGE,
    }
}

#[cfg(test)]
mod tests {
    use super::reader_body_style;
    use crate::reader::{BookFont, BookFontSize, ReadingTheme};

    #[test]
    fn resolves_all_reader_body_profiles() {
        for family in [
            BookFont::Inter,
            BookFont::AtkinsonHyperlegible,
            BookFont::Serif,
        ] {
            for size in [
                BookFontSize::Small,
                BookFontSize::Medium,
                BookFontSize::Large,
                BookFontSize::XLarge,
            ] {
                assert!(reader_body_style(family, size, ReadingTheme::Classic).line_height() > 0);
            }
        }
    }
}
