use ratatui::style::Color;

// --- Palette ---
pub(super) const fn c_bg() -> Color {
    Color::Rgb(24, 26, 33)
}
pub(super) const fn c_bg_panel() -> Color {
    Color::Rgb(24, 26, 33)
}
pub(super) const fn c_border() -> Color {
    Color::Gray
}
pub(super) const fn c_accent() -> Color {
    Color::Green
} // cyan-ish
pub(super) const fn c_accent2() -> Color {
    Color::Yellow
} // purple
pub(super) const fn c_ok() -> Color {
    Color::Rgb(120, 220, 120)
}
pub(super) const fn c_warn() -> Color {
    Color::LightYellow
}
pub(super) const fn c_err() -> Color {
    Color::Red
}
pub(super) const fn c_text() -> Color {
    Color::Rgb(220, 224, 232)
}
pub(super) const fn c_text_dim() -> Color {
    Color::Rgb(140, 145, 160)
}
