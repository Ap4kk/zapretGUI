use crate::app::App;
use crate::core::diagnostics::{self, CheckStatus};
use crate::theme::{self, Palette};
use crate::ui::widgets;
use eframe::egui::{self, Color32, RichText};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    widgets::section_header(
        ui,
        "Diagnostics",
        Some("Detect common issues that prevent zapret from working"),
    );

    ui.horizontal(|ui| {
        if ui.add(theme::accent_button("▶  Run all checks")).clicked() {
            app.run_diagnostics();
        }
        if ui.add(theme::ghost_button("Fix TCP timestamps")).clicked() {
            match diagnostics::fix_tcp_timestamps() {
                Ok(_) => app.set_toast("Timestamps enabled".into()),
                Err(e) => app.set_toast(format!("Fix failed: {}", e)),
            }
        }
    });
    ui.add_space(8.0);

    if app.diag_running {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(RichText::new("Running checks...").color(Palette::TEXT_MUTED));
        });
        ui.add_space(8.0);
    }

    egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
        if app.diag_results.is_empty() {
            widgets::card(ui, |ui| {
                ui.label(
                    RichText::new("Click \"Run all checks\" to begin.")
                        .color(Palette::TEXT_MUTED),
                );
            });
        } else {
            for r in &app.diag_results.clone() {
                widgets::card(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (color, icon) = match r.status {
                            CheckStatus::Pass => (Palette::SUCCESS, "✓"),
                            CheckStatus::Warn => (Palette::WARNING, "!"),
                            CheckStatus::Fail => (Palette::ERROR, "✕"),
                            CheckStatus::Info => (Palette::TEXT_MUTED, "i"),
                        };
                        circle_badge(ui, icon, color);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&r.name).strong().size(14.0));
                            ui.label(RichText::new(&r.message).color(Palette::TEXT_MUTED));
                        });
                    });
                });
                ui.add_space(6.0);
            }
        }
    });
}

fn circle_badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    let size = egui::vec2(22.0, 22.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();
    painter.circle_filled(rect.center(), 10.0, color);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(13.0),
        Color32::WHITE,
    );
}
