#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]

mod app;
mod theme;
mod core;
mod ui;

use eframe::egui;

fn main() -> eframe::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let icon = load_icon();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1120.0, 720.0])
        .with_min_inner_size([900.0, 560.0])
        .with_title("Zapret GUI");

    if let Some(ic) = icon {
        viewport = viewport.with_icon(ic);
    }

    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Zapret GUI",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            Ok(Box::new(app::App::new(cc)))
        }),
    )
}

fn load_icon() -> Option<egui::IconData> {
    // Fallback: generate a simple violet square icon at runtime if no .ico is bundled.
    let size = 64u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let cx = x as f32 / size as f32 - 0.5;
            let cy = y as f32 / size as f32 - 0.5;
            let d = (cx * cx + cy * cy).sqrt();
            if d < 0.48 {
                let t = (1.0 - d * 2.0).clamp(0.0, 1.0);
                let r = (0.486 + 0.2 * t) * 255.0;
                let g = (0.361 + 0.1 * t) * 255.0;
                let b = (1.0) * 255.0;
                rgba.extend_from_slice(&[r as u8, g as u8, b as u8, 255]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    Some(egui::IconData {
        rgba,
        width: size,
        height: size,
    })
}
