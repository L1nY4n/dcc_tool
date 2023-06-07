// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use dcc_tool::ui::DccTestApp;
use eframe::{epaint::Vec2, run_native, NativeOptions};
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .init();

    let native_options = NativeOptions {
        initial_window_size: Some(Vec2 { x: 600.0, y: 800.0 }),
        transparent: false,

        resizable: true,

        ..NativeOptions::default()
    };
    run_native(
        "Dcc Test",
        native_options,
        Box::new(|cc: &eframe::CreationContext| Box::new(DccTestApp::new(cc))),
    )
    .unwrap();
}
