use crate::app::App;
use crate::theme::{self, Palette};
use crate::ui::widgets;
use eframe::egui::{self, RichText};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    widgets::section_header(ui, "Logs", Some("Output from winws.exe and the GUI"));

    ui.horizontal(|ui| {
        if ui.add(theme::ghost_button("🗑 Clear")).clicked() {
            app.runner.logs.clear();
        }
        ui.checkbox(&mut app.log_autoscroll, "Auto-scroll");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{} lines", app.runner.logs.len()))
                    .color(Palette::TEXT_MUTED),
            );
        });
    });
    ui.add_space(6.0);

    let lines = app.runner.logs.snapshot();
    egui::Frame::none()
        .fill(Palette::PANEL_LO)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::same(8.0))
        .show(ui, |ui| {
            let mut scroll = egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(ui.available_height());
            if app.log_autoscroll {
                scroll = scroll.stick_to_bottom(true);
            }
            scroll.show(ui, |ui| {
                if lines.is_empty() {
                    ui.label(RichText::new("(no logs yet)").color(Palette::TEXT_MUTED));
                } else {
                    for l in &lines {
                        let color = if l.starts_with("[err]") {
                            Palette::ERROR
                        } else if l.starts_with('→') || l.starts_with('✓') {
                            Palette::ACCENT
                        } else {
                            Palette::TEXT
                        };
                        ui.label(RichText::new(l).monospace().color(color));
                    }
                }
            });
        });
}
