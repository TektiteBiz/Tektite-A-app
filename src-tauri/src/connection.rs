use serde::{Deserialize, Serialize};
use serialport::{available_ports, SerialPort, SerialPortType};
use std::{io::Read, mem, slice, sync::Mutex, thread::sleep, time::Duration};
use tauri::api::dialog;
use tauri::{Manager, Window};

pub struct SerialPortState(pub Mutex<Option<Box<dyn SerialPort>>>);

#[tauri::command]
pub fn connect(port: tauri::State<SerialPortState>) -> bool {
    let res = available_ports();
    if let Err(_) = res {
        return false;
    }
    let mut name: Option<String> = None;
    for port in res.unwrap() {
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
        .timeout(Duration::from_secs(5))
        .flow_control(serialport::FlowControl::Software)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open();
    if let Err(_) = &v {
        return false;
    }
    if let Err(_) = v.as_mut().unwrap().write_data_terminal_ready(true) {
        return false;
    }
    *port.0.lock().unwrap() = Some(v.unwrap());
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
    FlightReplay,
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
    starttime: u32,
    P: f32,
    alpha: f32,
    mass: f32,
}

#[repr(C, packed)]
pub struct Command {
    command_type: u8,
    config: Config,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Serialize, Default)]
pub struct StatusData {
    has_data: bool,
    config: Config,
}

fn handle_err(port: Option<Box<dyn SerialPort>>, app_handle: tauri::AppHandle) {
    dialog::message(None::<&Window>, "Tektite-A", "Disconnected from device");
    app_handle
        .emit_all("disconnect", "")
        .expect("Failed to communicate with frontend");
    mem::drop(port);
}

#[tauri::command]
pub fn get_status(port: tauri::State<SerialPortState>, app_handle: tauri::AppHandle) -> StatusData {
    let mut binding = port.0.lock().unwrap();
    if !binding.is_some() {
        return StatusData::default();
    }
    let mut val = binding.as_mut().unwrap();
    if let Err(_) = val.clear(serialport::ClearBuffer::Input) {
        handle_err(binding.take(), app_handle);
        return StatusData::default();
    }
    send_command(
        &mut val,
        Command {
            command_type: CommandType::Status as u8,
            config: Config::default(),
        },
    );

    let mut status: StatusData = unsafe { mem::zeroed() };
    let mut buff: [u8; mem::size_of::<StatusData>()] = [0; mem::size_of::<StatusData>()];
    if let Err(_) = val.read_exact(&mut buff) {
        handle_err(binding.take(), app_handle);
        return StatusData::default();
    }

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
pub fn config_write(
    port: tauri::State<SerialPortState>,
    config: Config,
    app_handle: tauri::AppHandle,
) {
    let mut binding = port.0.lock().unwrap();
    let mut val = binding.as_mut().unwrap();
    if !send_command(
        &mut val,
        Command {
            command_type: CommandType::ConfigWrite as u8,
            config,
        },
    ) {
        handle_err(binding.take(), app_handle);
    }
}

#[tauri::command]
pub fn servo_test(
    port: tauri::State<SerialPortState>,
    config: Config,
    max: bool,
    app_handle: tauri::AppHandle,
) {
    let mut binding = port.0.lock().unwrap();
    let mut val = binding.as_mut().unwrap();
    if !send_command(
        &mut val,
        Command {
            command_type: if max {
                CommandType::ServoMax
            } else {
                CommandType::ServoMin
            } as u8,
            config,
        },
    ) {
        handle_err(binding.take(), app_handle);
    }
}

fn send_command(port: &mut Box<dyn serialport::SerialPort>, command: Command) -> bool {
    let data = unsafe {
        slice::from_raw_parts(
            &command as *const Command as *const u8,
            mem::size_of::<Command>(),
        )
    };
    if let Err(_) = port.write_all(data) {
        return false;
    }
    port.flush().expect("Failed to flush Serial port");
    true
}

#[repr(C, packed)]
#[derive(Serialize, Clone, Copy)]
pub struct Frame {
    time: u32,
    alt: f32,
    vz: f32,
    vx: f32,
    vy: f32,
    az: f32,
    pre: f32,
    servo: f32,

    ax: f32,
    ay: f32,

    axr: f32,
    ayr: f32,
    azr: f32,
    gxr: f32,
    gyr: f32,
    gzr: f32,

    target: f32,

    altr: f32,
    baro: f32,
    temp: f32,

    state: u8,
    samples: u8,
}

#[repr(C, packed)]
pub struct SensorBuf {
    zero: u32,
    sample_count: u8,
    buf: [Frame; 42],
}

#[tauri::command(async)]
pub fn read_data(port: tauri::State<SerialPortState>, app_handle: tauri::AppHandle, path: String) {
    let mut binding = port.0.lock().unwrap();
    let mut val = binding.as_mut().unwrap();

    // Prepare writer
    let mut writer = csv::Writer::from_path(path).expect("Failed to open CSV");
    if let Err(_) = val.clear(serialport::ClearBuffer::Input) {
        handle_err(binding.take(), app_handle);
        return;
    }
    if !send_command(
        &mut val,
        Command {
            command_type: CommandType::DataRead as u8,
            config: Config::default(),
        },
    ) {
        handle_err(binding.take(), app_handle);
        return;
    }

    // Read
    let mut zero = 0;
    while zero == 0 {
        let mut buf: SensorBuf = unsafe { mem::zeroed() };
        let mut buff: [u8; mem::size_of::<SensorBuf>()] = [0; mem::size_of::<SensorBuf>()];
        val.read_exact(&mut buff)
            .expect("Failed to read from Serial port");

        unsafe {
            let config_slice = slice::from_raw_parts_mut(
                &mut buf as *mut _ as *mut u8,
                mem::size_of::<SensorBuf>(),
            );
            // `read_exact()` comes from `Read` impl for `&[u8]`
            (&buff[0..buff.len()]).read_exact(config_slice).unwrap();
        }
        zero = buf.zero;
        if zero != 0 {
            break;
        }

        let mut t = 0;
        for i in 0..(buf.sample_count as usize) {
            writer
                .serialize(buf.buf[i])
                .expect("Failed to write to CSV");
            if buf.buf[i].time > t {
                t = buf.buf[i].time;
            }
        }
        app_handle
            .emit_all("recvdata", t)
            .expect("Failed to communicate with frontend");

        val.clear(serialport::ClearBuffer::Input)
            .expect("Failed to clear Serial port");
        val.write_all(&[1]).expect("Failed to write to Serial port");
    }
}

#[repr(C, packed)]
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct ReplayData {
    pub delay: i32,
    pub servo: f32,
}

#[tauri::command(async)]
pub fn play_flight(
    data: Vec<ReplayData>,
    port: tauri::State<SerialPortState>,
    app_handle: tauri::AppHandle,
) {
    let mut binding = port.0.lock().unwrap();
    let mut val = binding.as_mut().unwrap();
    if let Err(_) = val.clear(serialport::ClearBuffer::Input) {
        handle_err(binding.take(), app_handle);
        return;
    }
    if !send_command(
        &mut val,
        Command {
            command_type: CommandType::FlightReplay as u8,
            config: Config::default(),
        },
    ) {
        handle_err(binding.take(), app_handle);
        return;
    }

    for chunk in data.chunks(8) {
        // Wait for ack
        val.read_exact(&mut [0; 1])
            .expect("Failed to read from Serial port");

        // Send data
        let mut buf: [ReplayData; 8] = [ReplayData {
            delay: 0,
            servo: 0.0,
        }; 8];
        for i in 0..8 {
            if i < chunk.len() {
                buf[i] = chunk[i];
            } else {
                buf[i].delay = -1;
            }
        }
        let data = unsafe {
            slice::from_raw_parts(
                &buf as *const _ as *const u8,
                mem::size_of::<ReplayData>() * 8,
            )
        };
        val.write_all(data).expect("Failed to write to Serial port");
        val.flush().expect("Failed to flush Serial port");
    }
}
