use crate::app::App;
use crate::core::{service, settings};
use crate::theme::{self, Palette};
use crate::ui::widgets;
use eframe::egui::{self, RichText};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    widgets::section_header(
        ui,
        "Service",
        Some("Install zapret as a Windows service that starts at boot"),
    );

    let zapret_state = service::query_service(service::ZAPRET_SERVICE);
    let installed_strategy = service::current_installed_strategy();

    widgets::card(ui, |ui| {
        ui.label(RichText::new("Service status").color(Palette::TEXT_MUTED).size(11.0));
        ui.horizontal(|ui| {
            let (color, text) = match zapret_state {
                service::ServiceState::Running => (Palette::SUCCESS, "running"),
                service::ServiceState::Stopped => (Palette::WARNING, "stopped"),
                service::ServiceState::NotInstalled => (Palette::TEXT_MUTED, "not installed"),
                _ => (Palette::TEXT_MUTED, "transitional"),
            };
            widgets::icon_dot(ui, color);
            ui.label(RichText::new(text).strong());
            if let Some(s) = &installed_strategy {
                ui.label(RichText::new(format!("• strategy: {}", s)).color(Palette::TEXT_MUTED));
            }
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.add(theme::ghost_button("↻ Restart service")).clicked() {
                match service::restart_service() {
                    Ok(_) => app.set_toast("Service restarted".into()),
                    Err(e) => app.set_toast(format!("Restart failed: {}", e)),
                }
            }
            if ui.add(theme::danger_button("🗑 Remove service")).clicked() {
                match service::remove_service() {
                    Ok(_) => app.set_toast("Service removed".into()),
                    Err(e) => app.set_toast(format!("Remove failed: {}", e)),
                }
            }
        });
    });

    ui.add_space(12.0);

    widgets::card(ui, |ui| {
        ui.label(RichText::new("Toggles").strong());
        ui.add_space(6.0);

        let mut mode = settings::read_game_mode();
        ui.horizontal(|ui| {
            ui.label("Game Filter:");
            egui::ComboBox::from_id_source("game_mode")
                .selected_text(mode.label())
                .show_ui(ui, |ui| {
                    for m in [
                        settings::GameMode::Disabled,
                        settings::GameMode::Both,
                        settings::GameMode::TcpOnly,
                        settings::GameMode::UdpOnly,
                    ] {
                        if ui.selectable_value(&mut mode, m, m.label()).clicked() {}
                    }
                });
            if ui.add(theme::ghost_button("Apply")).clicked() {
                match settings::write_game_mode(mode) {
                    Ok(_) => app.set_toast("Game filter saved".into()),
                    Err(e) => app.set_toast(format!("Save failed: {}", e)),
                }
            }
        });
        ui.label(
            RichText::new("Opens TCP/UDP 1024-65535 to bypass DPI for online games")
                .color(Palette::TEXT_MUTED)
                .size(11.0),
        );

        ui.add_space(10.0);

        let mut auto = settings::read_auto_update();
        if ui.checkbox(&mut auto, "Check for updates on launch").clicked() {
            if let Err(e) = settings::write_auto_update(auto) {
                app.set_toast(format!("Save failed: {}", e));
            }
        }

        ui.add_space(10.0);

        let ipset_mode = settings::read_ipset_mode();
        ui.horizontal(|ui| {
            ui.label("IPSet Filter:");
            ui.label(RichText::new(ipset_mode.label()).strong());
            if ui.add(theme::ghost_button("Cycle mode")).clicked() {
                match settings::switch_ipset_mode() {
                    Ok(n) => app.set_toast(format!("IPSet → {}", n.label())),
                    Err(e) => app.set_toast(format!("Switch failed: {}", e)),
                }
            }
        });
        ui.label(
            RichText::new("loaded: real list • none: stub IP only • any: empty list (match all)")
                .color(Palette::TEXT_MUTED)
                .size(11.0),
        );
    });
}
