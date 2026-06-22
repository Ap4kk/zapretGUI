use crate::core::{bat_parser, diagnostics, service, settings, updates, winws};
use crate::theme::Palette;
use crate::ui::{
    page_diagnostics, page_lists, page_logs, page_service, page_settings, page_strategies,
    sidebar::{self, Page},
    widgets,
};
use crossbeam_channel::{unbounded, Receiver};
use eframe::egui::{self, RichText};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct App {
    pub current_page: Page,
    pub runner: winws::Runner,

    pub diag_results: Vec<diagnostics::CheckResult>,
    pub diag_running: bool,
    pub diag_rx: Option<Receiver<Vec<diagnostics::CheckResult>>>,

    pub remote_version: Option<String>,
    pub update_checking: bool,
    pub update_rx: Option<Receiver<UpdateMsg>>,

    pub selected_list: usize,
    pub list_buffer: String,
    pub list_dirty: bool,
    pub pending_hosts: Option<String>,

    pub log_autoscroll: bool,

    pub toast: Option<(String, Instant)>,
    pub preview: Option<(String, String)>, // (name, formatted command)

    pub install_rx: Option<Receiver<Result<String, String>>>,
    pub install_in_progress: bool,
}

pub enum UpdateMsg {
    Version(Result<String, String>),
    IpsetDone(Result<usize, String>),
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let _ = settings::ensure_user_lists();
        let initial_list = crate::core::lists::UserList::GeneralUser;
        let buf = crate::core::lists::read(initial_list).unwrap_or_default();
        Self {
            current_page: Page::Strategies,
            runner: winws::Runner::default(),
            diag_results: Vec::new(),
            diag_running: false,
            diag_rx: None,
            remote_version: None,
            update_checking: false,
            update_rx: None,
            selected_list: 0,
            list_buffer: buf,
            list_dirty: false,
            pending_hosts: None,
            log_autoscroll: true,
            toast: None,
            preview: None,
            install_rx: None,
            install_in_progress: false,
        }
    }

    pub fn set_toast(&mut self, msg: String) {
        self.toast = Some((msg, Instant::now()));
    }

    pub fn refresh_strategies(&mut self) {
        // Nothing to cache yet — the scan happens each frame. Force a repaint.
        self.set_toast("Refreshed".into());
    }

    pub fn install_service_for(&mut self, path: PathBuf, name: String) {
        let gf = bat_parser::GameFilter::from_settings(settings::read_game_mode());
        let parsed = match bat_parser::parse_bat(&path, gf) {
            Ok(p) => p,
            Err(e) => {
                self.set_toast(format!("Parse failed: {}", e));
                return;
            }
        };
        let (tx, rx) = unbounded::<Result<String, String>>();
        self.install_rx = Some(rx);
        self.install_in_progress = true;
        std::thread::spawn(move || {
            let res = service::install_service(&name, &parsed.args)
                .map(|_| format!("Service installed: {}", name))
                .map_err(|e| e.to_string());
            let _ = tx.send(res);
        });
    }

    pub fn show_strategy_preview(&mut self, path: PathBuf, name: String, gf: bat_parser::GameFilter) {
        match bat_parser::parse_bat(&path, gf) {
            Ok(p) => {
                let mut s = String::new();
                s.push_str(&crate::core::paths::winws_exe().to_string_lossy());
                for a in &p.args {
                    s.push(' ');
                    if a.contains(' ') {
                        s.push('"');
                        s.push_str(a);
                        s.push('"');
                    } else {
                        s.push_str(a);
                    }
                }
                self.preview = Some((name, s));
            }
            Err(e) => self.set_toast(format!("Parse failed: {}", e)),
        }
    }

    pub fn run_diagnostics(&mut self) {
        if self.diag_running {
            return;
        }
        self.diag_running = true;
        let (tx, rx) = unbounded();
        self.diag_rx = Some(rx);
        std::thread::spawn(move || {
            let r = diagnostics::run_all();
            let _ = tx.send(r);
        });
    }

    pub fn check_updates_async(&mut self) {
        if self.update_checking {
            return;
        }
        self.update_checking = true;
        let (tx, rx) = unbounded::<UpdateMsg>();
        self.update_rx = Some(rx);
        std::thread::spawn(move || {
            let v = updates::remote_version().map_err(|e| e.to_string());
            let _ = tx.send(UpdateMsg::Version(v));
        });
    }

    pub fn update_ipset_async(&mut self) {
        let (tx, rx) = unbounded::<UpdateMsg>();
        // Reuse the same receiver slot
        self.update_rx = Some(rx);
        std::thread::spawn(move || {
            let r = updates::update_ipset().map_err(|e| e.to_string());
            let _ = tx.send(UpdateMsg::IpsetDone(r));
        });
    }

    fn pump_background(&mut self) {
        if let Some(rx) = &self.diag_rx {
            if let Ok(r) = rx.try_recv() {
                self.diag_results = r;
                self.diag_running = false;
                self.diag_rx = None;
            }
        }
        if let Some(rx) = &self.update_rx {
            match rx.try_recv() {
                Ok(UpdateMsg::Version(Ok(v))) => {
                    self.remote_version = Some(v);
                    self.update_checking = false;
                    self.update_rx = None;
                    self.set_toast("Update check done".into());
                }
                Ok(UpdateMsg::Version(Err(e))) => {
                    self.update_checking = false;
                    self.update_rx = None;
                    self.set_toast(format!("Update check failed: {}", e));
                }
                Ok(UpdateMsg::IpsetDone(Ok(n))) => {
                    self.update_rx = None;
                    // Refresh editor if user is on the ipset tab (index 3 in ALL_LISTS).
                    if self.selected_list == 3 {
                        self.list_buffer =
                            crate::core::lists::read(crate::core::lists::UserList::IpsetAll)
                                .unwrap_or_default();
                        self.list_dirty = false;
                    }
                    self.set_toast(format!("IPSet updated: {} entries", n));
                }
                Ok(UpdateMsg::IpsetDone(Err(e))) => {
                    self.update_rx = None;
                    self.set_toast(format!("IPSet update failed: {}", e));
                }
                Err(_) => {}
            }
        }
        if let Some(rx) = &self.install_rx {
            if let Ok(res) = rx.try_recv() {
                self.install_in_progress = false;
                self.install_rx = None;
                match res {
                    Ok(s) => self.set_toast(s),
                    Err(e) => self.set_toast(format!("Install failed: {}", e)),
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.pump_background();

        let running = self.runner.is_running();
        let log_count = self.runner.logs.len();

        egui::SidePanel::left("sidebar")
            .resizable(false)
            .show_separator_line(false)
            .show(ctx, |ui| {
                sidebar::show(ui, &mut self.current_page, running, log_count);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_page {
                Page::Strategies => page_strategies::show(self, ui),
                Page::Service => page_service::show(self, ui),
                Page::Diagnostics => page_diagnostics::show(self, ui),
                Page::Lists => page_lists::show(self, ui),
                Page::Settings => page_settings::show(self, ui),
                Page::Logs => page_logs::show(self, ui),
            }
        });

        // Preview window
        if let Some((name, cmd)) = self.preview.clone() {
            let mut open = true;
            egui::Window::new(format!("Command: {}", name))
                .open(&mut open)
                .default_width(720.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut cmd.clone())
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .desired_rows(16),
                        );
                    });
                    ui.horizontal(|ui| {
                        if ui.add(crate::theme::ghost_button("Copy")).clicked() {
                            ctx.output_mut(|o| o.copied_text = cmd.clone());
                        }
                    });
                });
            if !open {
                self.preview = None;
            }
        }

        // Install progress overlay
        if self.install_in_progress {
            egui::Window::new("Installing service")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Working...");
                    });
                });
        }

        // Toast
        if let Some((msg, started)) = self.toast.clone() {
            if started.elapsed() < Duration::from_secs(4) {
                egui::Area::new(egui::Id::new("toast"))
                    .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -24.0])
                    .show(ctx, |ui| {
                        egui::Frame::none()
                            .fill(Palette::PANEL_HI)
                            .stroke(egui::Stroke::new(1.0, Palette::ACCENT))
                            .rounding(egui::Rounding::same(10.0))
                            .inner_margin(egui::Margin::symmetric(14.0, 8.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(&msg).color(Palette::TEXT));
                            });
                    });
            } else {
                self.toast = None;
            }
        }

        // Keep ticking so timers/log streams refresh.
        ctx.request_repaint_after(Duration::from_millis(250));

        // Use widgets module path to avoid dead_code warnings.
        let _ = widgets::link_button;
    }
}
