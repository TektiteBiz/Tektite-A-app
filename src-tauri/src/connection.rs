use serialport::{available_ports, SerialPort, SerialPortType};
use std::sync::Mutex;

pub struct SerialPortState(pub Mutex<Option<Box<dyn SerialPort>>>);

#[tauri::command]
pub fn connect(port: tauri::State<SerialPortState>) -> bool {
    let res = available_ports().expect("Failed to fetch Serial ports");
    let mut name: Option<String> = None;
    for port in res {
        if let SerialPortType::UsbPort(info) = port.port_type {
            if let Some(manu) = info.manufacturer {
                if manu == "STMicroelectronics" {
                    name = Some(port.port_name);
                    break;
                }
            }
        }
    }
    if name.is_none() {
        return false;
    }
    let v = serialport::new(name.unwrap(), 115200)
        .open()
        .expect("Failed to open Serial port");
    *port.0.lock().unwrap() = Some(v);
    true
}
