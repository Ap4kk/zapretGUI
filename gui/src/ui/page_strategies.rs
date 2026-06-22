use crate::app::App;
use crate::core::{bat_parser, paths, settings};
use crate::theme::{self, Palette};
use crate::ui::widgets;
use eframe::egui::{self, RichText};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    widgets::section_header(ui, "Strategies", Some("Pick a winws.exe strategy and run it"));

    ui.horizontal(|ui| {
        let running = app.runner.is_running();
        if running {
            let name = app
                .runner
                .current_strategy
                .clone()
                .unwrap_or_else(|| "(unknown)".to_string());
            widgets::status_pill(ui, &format!("RUNNING — {}", name), Palette::SUCCESS);
            if ui.add(theme::danger_button("■  Stop")).clicked() {
                if let Err(e) = app.runner.stop() {
                    app.set_toast(format!("Stop failed: {}", e));
                } else {
                    app.set_toast("Stopped".into());
                }
            }
        } else {
            widgets::status_pill(ui, "STOPPED", Palette::TEXT_MUTED);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(theme::ghost_button("↻ Refresh list")).clicked() {
                app.refresh_strategies();
            }
        });
    });

    ui.add_space(8.0);

    let bats = paths::strategy_bats();
    if bats.is_empty() {
        widgets::card(ui, |ui| {
            ui.label(RichText::new("No general*.bat files found in the zapret root directory.").color(Palette::WARNING));
            ui.label(
                RichText::new(format!("Looked in: {}", paths::root_dir().display()))
                    .color(Palette::TEXT_MUTED),
            );
        });
        return;
    }

    let gf = bat_parser::GameFilter::from_settings(settings::read_game_mode());

    egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
        let available = ui.available_width();
        let card_w = 340.0_f32;
        let cols = ((available / (card_w + 12.0)).floor() as usize).max(1);
        let row_w = (available - (cols - 1) as f32 * 12.0) / cols as f32;

        let mut i = 0;
        while i < bats.len() {
            ui.horizontal_wrapped(|ui| {
                for _ in 0..cols {
                    if i >= bats.len() {
                        break;
                    }
                    let path = &bats[i];
                    let name = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "?".to_string());

                    ui.allocate_ui(egui::vec2(row_w, 0.0), |ui| {
                        widgets::card(ui, |ui| {
                            ui.set_width(ui.available_width());
                            let is_current = app
                                .runner
                                .current_strategy
                                .as_deref()
                                .map(|c| c == name)
                                .unwrap_or(false);

                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&name).strong().size(15.0));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if is_current {
                                            widgets::status_pill(ui, "active", Palette::SUCCESS);
                                        }
                                    },
                                );
                            });
                            ui.label(
                                RichText::new(short_path(&path.to_string_lossy()))
                                    .color(Palette::TEXT_MUTED)
                                    .size(11.0),
                            );
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                let run_btn = if is_current {
                                    ui.add_enabled(false, theme::accent_button("► Run"))
                                } else {
                                    ui.add(theme::accent_button("► Run"))
                                };
                                if run_btn.clicked() {
                                    match bat_parser::parse_bat(path, gf) {
                                        Ok(strat) => {
                                            if app.runner.is_running() {
                                                let _ = app.runner.stop();
                                            }
                                            match app.runner.start(&name, &strat.args) {
                                                Ok(_) => app.set_toast(format!("Running {}", name)),
                                                Err(e) => {
                                                    app.set_toast(format!("Failed: {}", e))
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            app.set_toast(format!("Parse error: {}", e));
                                        }
                                    }
                                }
                                if ui.add(theme::ghost_button("⚙ Install as service")).clicked() {
                                    app.install_service_for(path.clone(), name.clone());
                                }
                                if ui.add(theme::ghost_button("👁 View")).clicked() {
                                    app.show_strategy_preview(path.clone(), name.clone(), gf);
                                }
                            });
                        });
                    });
                    i += 1;
                }
            });
            ui.add_space(8.0);
        }
    });
}

fn short_path(s: &str) -> String {
    let p = std::path::Path::new(s);
    p.file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| s.to_string())
}
