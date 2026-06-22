use crate::theme::Palette;
use eframe::egui::{self, Color32, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};

pub fn section_header(ui: &mut Ui, title: &str, subtitle: Option<&str>) {
    ui.add_space(4.0);
    ui.label(RichText::new(title).heading().color(Palette::TEXT));
    if let Some(s) = subtitle {
        ui.label(RichText::new(s).color(Palette::TEXT_MUTED));
    }
    ui.add_space(10.0);
    ui.separator();
    ui.add_space(8.0);
}

pub fn status_pill(ui: &mut Ui, text: &str, color: Color32) {
    let label = RichText::new(text).color(Color32::WHITE).strong();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(0.0, 22.0), Sense::hover());
    let _ = rect;
    egui::Frame::none()
        .fill(color)
        .rounding(Rounding::same(11.0))
        .inner_margin(egui::Margin::symmetric(10.0, 3.0))
        .show(ui, |ui| {
            ui.label(label);
        });
}

pub fn card<R>(ui: &mut Ui, contents: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::none()
        .fill(Palette::PANEL)
        .stroke(Stroke::new(1.0, Palette::BORDER))
        .rounding(Rounding::same(10.0))
        .inner_margin(egui::Margin::same(14.0))
        .show(ui, contents)
        .inner
}

pub fn icon_dot(ui: &mut Ui, color: Color32) -> Response {
    let size = Vec2::splat(10.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    painter.circle_filled(rect.center(), 5.0, color);
    resp
}

pub fn link_button(ui: &mut Ui, text: &str) -> Response {
    ui.add(egui::Button::new(RichText::new(text).color(Palette::ACCENT))
        .frame(false))
}
