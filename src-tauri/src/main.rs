#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod ffmpeg;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::probe_video,
            commands::export_package,
            commands::open_kofi
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
