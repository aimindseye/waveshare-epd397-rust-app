//! Product identity and semantic firmware metadata.
//!
//! Keep product-facing metadata in one place so UI screens and serial markers
//! report the same values. The ESP-IDF application version is also pinned in
//! `sdkconfig.defaults` for the bootloader application descriptor.

/// Human-readable product label rendered in the UI.
pub const PRODUCT_NAME: &str = "Rustmix Wave / EPD397";
/// Stable machine-readable product identifier used in serial markers.
pub const PRODUCT_SLUG: &str = "rustmix-wave-epd397";
/// Cargo semantic version for the current firmware package.
pub const FIRMWARE_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Stable milestone identifier for acceptance logs and diagnostics.
pub const UI_SHELL_MILESTONE: &str = "unit-converter-foundation";

#[cfg(test)]
mod tests {
    use super::{FIRMWARE_VERSION, PRODUCT_NAME, PRODUCT_SLUG, UI_SHELL_MILESTONE};

    #[test]
    fn exposes_unit_converter_foundation_metadata() {
        assert_eq!(PRODUCT_NAME, "Rustmix Wave / EPD397");
        assert_eq!(PRODUCT_SLUG, "rustmix-wave-epd397");
        assert_eq!(FIRMWARE_VERSION, "0.15.0");
        assert_eq!(UI_SHELL_MILESTONE, "unit-converter-foundation");
    }
}
