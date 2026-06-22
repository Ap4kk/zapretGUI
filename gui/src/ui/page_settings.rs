use crate::app::App;
use crate::core::{paths, updates};
use crate::theme::{self, Palette};
use crate::ui::widgets;
use eframe::egui::{self, RichText};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    widgets::section_header(ui, "Settings", Some("App configuration and update checks"));

    widgets::card(ui, |ui| {
        ui.label(RichText::new("Paths").strong());
        ui.add_space(4.0);
        path_row(ui, "Root:", &paths::root_dir().to_string_lossy());
        path_row(ui, "winws.exe:", &paths::winws_exe().to_string_lossy());
        path_row(ui, "lists/:", &paths::lists_dir().to_string_lossy());
        path_row(ui, "utils/:", &paths::utils_dir().to_string_lossy());
    });

    ui.add_space(10.0);

    widgets::card(ui, |ui| {
        ui.label(RichText::new("Updates").strong());
        ui.add_space(4.0);

        let local = updates::local_version().unwrap_or_else(|| "unknown".into());
        ui.label(format!("Installed zapret version: {}", local));
        if let Some(remote) = &app.remote_version {
            let color = if remote.as_str() != local {
                Palette::WARNING
            } else {
                Palette::SUCCESS
            };
            ui.label(RichText::new(format!("Latest on GitHub: {}", remote)).color(color));
        }
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            if ui.add(theme::accent_button("⤓ Check for updates")).clicked() {
                app.check_updates_async();
            }
            if app.update_checking {
                ui.spinner();
                ui.label(RichText::new("checking...").color(Palette::TEXT_MUTED));
            }
        });
    });

    ui.add_space(10.0);

    widgets::card(ui, |ui| {
        ui.label(RichText::new("About").strong());
        ui.label(
            RichText::new(format!(
                "Zapret GUI v{} — modern interface for Flowseal/zapret-discord-youtube",
                env!("CARGO_PKG_VERSION")
            ))
            .color(Palette::TEXT_MUTED),
        );
    });
}

fn path_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(Palette::TEXT_MUTED));
        ui.label(RichText::new(value).monospace());
    });
}
