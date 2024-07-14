// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod connection;
use connection::{
    config_write, connect, disconnect, get_status, is_connected, play_flight, read_data, servo_test,
};

mod sim;
use sim::calc_sim;

mod fs;
use fs::{read_flight_data, show_item_in_folder};

fn main() {
    tauri::Builder::default()
        .manage(connection::SerialPortState(std::sync::Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            connect,
            disconnect,
            is_connected,
            get_status,
            config_write,
            read_data,
            calc_sim,
            servo_test,
            read_flight_data,
            show_item_in_folder,
            play_flight
        ])
        .setup(|app| {
            #[cfg(target_os = "linux")]
            app.manage(DbusState(Mutex::new(
                dbus::blocking::SyncConnection::new_session().ok(),
            )));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
