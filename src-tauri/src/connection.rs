use serde::{Deserialize, Serialize};
use serialport::{available_ports, SerialPort, SerialPortType};
use std::{io::Read, mem, slice, sync::Mutex, thread::sleep, time::Duration};

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
    let mut v = serialport::new(name.unwrap(), 9600)
        .timeout(Duration::from_secs(30))
        .flow_control(serialport::FlowControl::Software)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open()
        .expect("Failed to open Serial port");
    v.write_data_terminal_ready(true)
        .expect("Failed to set DTR");
    *port.0.lock().unwrap() = Some(v);
    true
}

#[tauri::command]
pub fn is_connected(port: tauri::State<SerialPortState>) -> bool {
    port.0.lock().unwrap().is_some()
}

#[tauri::command]
pub fn disconnect(port: tauri::State<SerialPortState>) {
    mem::drop(port.0.lock().unwrap().take());
}

#[derive(Copy, Clone)]
pub enum CommandType {
    ServoMin,
    ServoMax,
    Status,
    ConfigWrite,
    DataRead,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Default, Serialize, Deserialize, Debug)]
pub struct Config {
    init: u32,
    s1min: i32,
    s2min: i32,
    s3min: i32,
    s1max: i32,
    s2max: i32,
    s3max: i32,
    control: bool,
    param: f32,
    burntime: u32,
    alpha: f32,
    mass: f32,
}

#[repr(C, packed)]
pub struct Command {
    command_type: u8,
    config: Config,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Serialize)]
pub struct StatusData {
    has_data: bool,
    config: Config,
}

#[tauri::command]
pub fn get_status(port: tauri::State<SerialPortState>) -> StatusData {
    let mut binding = port.0.lock().unwrap();
    let mut val = binding.as_mut().unwrap();
    val.clear(serialport::ClearBuffer::Input)
        .expect("Failed to clear Serial port");
    send_command(
        &mut val,
        Command {
            command_type: CommandType::Status as u8,
            config: Config::default(),
        },
    );

    let mut status: StatusData = unsafe { mem::zeroed() };
    let mut buff: [u8; mem::size_of::<StatusData>()] = [0; mem::size_of::<StatusData>()];
    val.read_exact(&mut buff)
        .expect("Failed to read from Serial port");

    unsafe {
        let config_slice = slice::from_raw_parts_mut(
            &mut status as *mut _ as *mut u8,
            mem::size_of::<StatusData>(),
        );
        // `read_exact()` comes from `Read` impl for `&[u8]`
        (&buff[0..buff.len()]).read_exact(config_slice).unwrap();
    }
    status
}

#[tauri::command]
pub fn config_write(port: tauri::State<SerialPortState>, config: Config) {
    println!("{:#?}", config);
    let mut binding = port.0.lock().unwrap();
    let mut val = binding.as_mut().unwrap();
    println!(
        "{} {}",
        std::mem::size_of::<Command>(),
        std::mem::size_of::<Config>()
    );
    send_command(
        &mut val,
        Command {
            command_type: CommandType::ConfigWrite as u8,
            config,
        },
    );
}

fn send_command(port: &mut Box<dyn serialport::SerialPort>, command: Command) {
    let data = unsafe {
        slice::from_raw_parts(
            &command as *const Command as *const u8,
            mem::size_of::<Command>(),
        )
    };
    port.write_all(data)
        .expect("Failed to write to Serial port");
    port.flush().expect("Failed to flush Serial port");
}
