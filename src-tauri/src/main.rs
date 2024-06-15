// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod connection;

fn main() {
    tauri::Builder::default()
        .manage(connection::SerialPortState(std::sync::Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![connection::connect])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
