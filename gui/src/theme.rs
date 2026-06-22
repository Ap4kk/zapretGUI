use eframe::egui::{self, Color32, FontFamily, FontId, Rounding, Stroke, Style, TextStyle, Visuals};

pub struct Palette;

#[allow(dead_code)]
impl Palette {
    pub const BG: Color32 = Color32::from_rgb(0x1B, 0x1C, 0x22);
    pub const PANEL: Color32 = Color32::from_rgb(0x24, 0x25, 0x2D);
    pub const PANEL_HI: Color32 = Color32::from_rgb(0x2E, 0x2F, 0x38);
    pub const PANEL_LO: Color32 = Color32::from_rgb(0x1F, 0x20, 0x27);
    pub const BORDER: Color32 = Color32::from_rgb(0x35, 0x37, 0x42);
    pub const TEXT: Color32 = Color32::from_rgb(0xE6, 0xE7, 0xEC);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x9A, 0x9D, 0xA8);
    pub const ACCENT: Color32 = Color32::from_rgb(0x7C, 0x5C, 0xFF);
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0x8E, 0x73, 0xFF);
    pub const ACCENT_ACTIVE: Color32 = Color32::from_rgb(0x6A, 0x48, 0xEE);
    pub const SUCCESS: Color32 = Color32::from_rgb(0x3F, 0xB9, 0x50);
    pub const WARNING: Color32 = Color32::from_rgb(0xD2, 0x99, 0x22);
    pub const ERROR: Color32 = Color32::from_rgb(0xF8, 0x51, 0x49);
}

pub fn install(ctx: &egui::Context) {
    let mut style: Style = (*ctx.style()).clone();

    style.text_styles = [
        (TextStyle::Heading, FontId::new(22.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
    ]
    .into();

    let mut v = Visuals::dark();
    v.override_text_color = Some(Palette::TEXT);
    v.window_fill = Palette::BG;
    v.panel_fill = Palette::BG;
    v.extreme_bg_color = Palette::PANEL_LO;
    v.faint_bg_color = Palette::PANEL;
    v.code_bg_color = Palette::PANEL_LO;
    v.window_stroke = Stroke::new(1.0, Palette::BORDER);
    v.window_rounding = Rounding::same(10.0);
    v.menu_rounding = Rounding::same(8.0);
    v.popup_shadow.color = Color32::from_black_alpha(140);

    // Widget styles
    v.widgets.noninteractive.bg_fill = Palette::PANEL;
    v.widgets.noninteractive.weak_bg_fill = Palette::PANEL;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Palette::BORDER);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Palette::TEXT);
    v.widgets.noninteractive.rounding = Rounding::same(8.0);

    v.widgets.inactive.bg_fill = Palette::PANEL_HI;
    v.widgets.inactive.weak_bg_fill = Palette::PANEL_HI;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, Palette::BORDER);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, Palette::TEXT);
    v.widgets.inactive.rounding = Rounding::same(8.0);

    v.widgets.hovered.bg_fill = Color32::from_rgb(0x38, 0x3A, 0x46);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x38, 0x3A, 0x46);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, Palette::ACCENT);
    v.widgets.hovered.fg_stroke = Stroke::new(1.5, Palette::TEXT);
    v.widgets.hovered.rounding = Rounding::same(8.0);

    v.widgets.active.bg_fill = Palette::ACCENT_ACTIVE;
    v.widgets.active.weak_bg_fill = Palette::ACCENT_ACTIVE;
    v.widgets.active.bg_stroke = Stroke::new(1.0, Palette::ACCENT_ACTIVE);
    v.widgets.active.fg_stroke = Stroke::new(1.5, Palette::TEXT);
    v.widgets.active.rounding = Rounding::same(8.0);

    v.widgets.open.bg_fill = Palette::PANEL_HI;
    v.widgets.open.weak_bg_fill = Palette::PANEL_HI;
    v.widgets.open.bg_stroke = Stroke::new(1.0, Palette::ACCENT);
    v.widgets.open.fg_stroke = Stroke::new(1.0, Palette::TEXT);
    v.widgets.open.rounding = Rounding::same(8.0);

    v.selection.bg_fill = Palette::ACCENT.linear_multiply(0.55);
    v.selection.stroke = Stroke::new(1.0, Palette::ACCENT);

    style.visuals = v;
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12.0);
    style.spacing.interact_size.y = 28.0;

    ctx.set_style(style);
}

pub fn accent_button(text: impl Into<egui::WidgetText>) -> egui::Button<'static> {
    egui::Button::new(text.into())
        .fill(Palette::ACCENT)
        .stroke(Stroke::new(1.0, Palette::ACCENT))
        .rounding(Rounding::same(8.0))
}

pub fn danger_button(text: impl Into<egui::WidgetText>) -> egui::Button<'static> {
    egui::Button::new(text.into())
        .fill(Palette::ERROR)
        .stroke(Stroke::new(1.0, Palette::ERROR))
        .rounding(Rounding::same(8.0))
}

pub fn ghost_button(text: impl Into<egui::WidgetText>) -> egui::Button<'static> {
    egui::Button::new(text.into())
        .fill(Palette::PANEL_HI)
        .stroke(Stroke::new(1.0, Palette::BORDER))
        .rounding(Rounding::same(8.0))
}
