use serialport::{available_ports, SerialPort};

#[tauri::command]
fn connect(port: tauri::State<Box<dyn SerialPort>>) {
    let res = available_ports().expect("Failed to fetch Serial ports");
    println!("{:?}", res);
    state = serialport::new("/test", 115200).open().expect("Failed to open Serial port");
    
}