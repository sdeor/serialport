fn main() {
    match serialport::available_ports() {
        Ok(ports) => {
            println!("Available ports:");
            for port in ports {
                println!("  {} ({})", port.port_name, {
                    match port.port_type {
                        serialport::config::SerialPortType::UsbPort(info) => {
                            format!("USB: {} {}", info.vid, info.pid)
                        }
                        serialport::config::SerialPortType::BluetoothPort => {
                            "Bluetooth".to_string()
                        }
                        serialport::config::SerialPortType::PciPort => "PCI".to_string(),
                        serialport::config::SerialPortType::Unknown => "Unknown".to_string(),
                    }
                });
            }
        }
        Err(e) => eprintln!("Failed to list ports: {}", e),
    }
}
