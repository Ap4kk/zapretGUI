use crate::app::App;
use crate::core::lists::{self, UserList};
use crate::core::updates;
use crate::theme::{self, Palette};
use crate::ui::widgets;
use eframe::egui::{self, RichText};

const ALL_LISTS: &[UserList] = &[
    UserList::GeneralUser,
    UserList::ExcludeUser,
    UserList::IpsetExcludeUser,
    UserList::IpsetAll,
];

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    widgets::section_header(
        ui,
        "User lists",
        Some("Domains and IP ranges that customize the bypass"),
    );

    ui.horizontal(|ui| {
        for (idx, list) in ALL_LISTS.iter().enumerate() {
            let selected = app.selected_list == idx;
            let resp = ui.add(if selected {
                theme::accent_button(list.label())
            } else {
                theme::ghost_button(list.label())
            });
            if resp.clicked() {
                app.selected_list = idx;
                app.list_buffer = lists::read(*list).unwrap_or_default();
                app.list_dirty = false;
            }
        }
    });

    let current = ALL_LISTS
        .get(app.selected_list)
        .copied()
        .unwrap_or(UserList::GeneralUser);

    ui.add_space(8.0);

    widgets::card(ui, |ui| {
        ui.label(RichText::new(current.description()).color(Palette::TEXT_MUTED));
        ui.label(
            RichText::new(format!("{}", current.path().display()))
                .color(Palette::TEXT_MUTED)
                .size(11.0),
        );
        ui.add_space(8.0);

        let prev = app.list_buffer.clone();
        let edit = egui::TextEdit::multiline(&mut app.list_buffer)
            .code_editor()
            .desired_rows(18)
            .desired_width(f32::INFINITY);
        ui.add(edit);
        if app.list_buffer != prev {
            app.list_dirty = true;
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.add(theme::accent_button("💾 Save")).clicked() {
                match lists::write(current, &app.list_buffer) {
                    Ok(_) => {
                        app.list_dirty = false;
                        app.set_toast("Saved".into());
                    }
                    Err(e) => app.set_toast(format!("Save failed: {}", e)),
                }
            }
            if ui.add(theme::ghost_button("⟲ Reload")).clicked() {
                app.list_buffer = lists::read(current).unwrap_or_default();
                app.list_dirty = false;
            }
            if matches!(current, UserList::IpsetAll) {
                if ui.add(theme::ghost_button("⤓ Update from repo")).clicked() {
                    app.update_ipset_async();
                }
            }
            if app.list_dirty {
                ui.label(RichText::new("● unsaved").color(Palette::WARNING));
            }
        });
    });

    ui.add_space(8.0);
    widgets::card(ui, |ui| {
        ui.label(RichText::new("Hosts file").strong());
        ui.label(
            RichText::new("Compare and update the system hosts overlay used by zapret")
                .color(Palette::TEXT_MUTED)
                .size(12.0),
        );
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.add(theme::ghost_button("Compare with repo")).clicked() {
                match (updates::fetch_remote_hosts(), updates::read_local_hosts()) {
                    (Ok(r), Ok(l)) => {
                        if updates::hosts_differs(&r, &l) {
                            app.set_toast("Hosts differs from repo".into());
                            app.pending_hosts = Some(r);
                        } else {
                            app.set_toast("Hosts is in sync".into());
                        }
                    }
                    _ => app.set_toast("Could not compare hosts".into()),
                }
            }
            if app.pending_hosts.is_some()
                && ui.add(theme::accent_button("Apply repo hosts")).clicked()
            {
                if let Some(h) = app.pending_hosts.take() {
                    match updates::write_local_hosts(&h) {
                        Ok(_) => app.set_toast("Hosts updated".into()),
                        Err(e) => app.set_toast(format!("Write failed: {}", e)),
                    }
                }
            }
        });
    });
}
