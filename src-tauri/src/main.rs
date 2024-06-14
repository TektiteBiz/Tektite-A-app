// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod connection;

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .invoke_handler(tauri::generate_handler![connection::connect])
        .expect("error while running tauri application");
}
