// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod connection;
use connection::{
    config_write, connect, disconnect, get_status, is_connected, read_data, servo_test,
};

mod sim;
use sim::calc_sim;

mod fs;
use fs::read_flight_data;

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
            read_flight_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
