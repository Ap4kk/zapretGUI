use crate::theme::Palette;
use eframe::egui::{self, Color32, RichText, Rounding, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Strategies,
    Service,
    Diagnostics,
    Lists,
    Settings,
    Logs,
}

impl Page {
    pub const ALL: &'static [Page] = &[
        Page::Strategies,
        Page::Service,
        Page::Diagnostics,
        Page::Lists,
        Page::Settings,
        Page::Logs,
    ];

    fn icon(self) -> &'static str {
        match self {
            Page::Strategies => "⚡",
            Page::Service => "⚙",
            Page::Diagnostics => "🩺",
            Page::Lists => "📝",
            Page::Settings => "🛠",
            Page::Logs => "📜",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Page::Strategies => "Strategies",
            Page::Service => "Service",
            Page::Diagnostics => "Diagnostics",
            Page::Lists => "Lists",
            Page::Settings => "Settings",
            Page::Logs => "Logs",
        }
    }
}

pub fn show(ui: &mut egui::Ui, current: &mut Page, running: bool, log_count: usize) {
    egui::Frame::none()
        .fill(Palette::PANEL_LO)
        .inner_margin(egui::Margin::symmetric(12.0, 16.0))
        .show(ui, |ui| {
            ui.set_min_width(196.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    egui::Frame::none()
                        .fill(Palette::ACCENT)
                        .rounding(Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("Z").color(Color32::WHITE).strong());
                        });
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Zapret GUI").strong().size(15.0));
                        ui.label(RichText::new("Discord & YouTube").color(Palette::TEXT_MUTED).size(11.0));
                    });
                });
                ui.add_space(16.0);

                for page in Page::ALL {
                    let selected = *current == *page;
                    let bg = if selected { Palette::ACCENT } else { Color32::TRANSPARENT };
                    let fg = if selected { Color32::WHITE } else { Palette::TEXT };

                    let resp = egui::Frame::none()
                        .fill(bg)
                        .rounding(Rounding::same(8.0))
                        .stroke(if selected {
                            Stroke::new(1.0, Palette::ACCENT)
                        } else {
                            Stroke::NONE
                        })
                        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(page.icon()).color(fg).size(14.0));
                                ui.label(RichText::new(page.label()).color(fg).size(14.0));
                                if *page == Page::Logs && log_count > 0 {
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            egui::Frame::none()
                                                .fill(if selected { Color32::from_white_alpha(40) } else { Palette::PANEL_HI })
                                                .rounding(Rounding::same(10.0))
                                                .inner_margin(egui::Margin::symmetric(6.0, 1.0))
                                                .show(ui, |ui| {
                                                    ui.label(
                                                        RichText::new(format!("{}", log_count))
                                                            .color(fg)
                                                            .size(11.0),
                                                    );
                                                });
                                        },
                                    );
                                }
                            });
                        })
                        .response
                        .interact(egui::Sense::click());
                    if resp.clicked() {
                        *current = *page;
                    }
                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    ui.add_space(2.0);
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(10.0);

                ui.label(RichText::new("STATUS").size(10.0).color(Palette::TEXT_MUTED));
                ui.horizontal(|ui| {
                    let (color, text) = if running {
                        (Palette::SUCCESS, "winws.exe running")
                    } else {
                        (Palette::TEXT_MUTED, "winws.exe stopped")
                    };
                    super::widgets::icon_dot(ui, color);
                    ui.label(RichText::new(text).size(12.0));
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .color(Palette::TEXT_MUTED)
                            .size(10.0),
                    );
                });

                let _ = Vec2::ZERO;
            });
        });
}
