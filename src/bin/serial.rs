fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::ErrorKind;

    use serialport::SerialPortBuilder;

    // Create a new serial port builder
    let builder = SerialPortBuilder::new().port("COM1").baud_rate(115200);

    let mut port = builder.build()?;
    port.close()?;

    match port.open() {
        Ok(()) => println!("Port opened successfully"),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => println!("Port not found"),
            ErrorKind::PermissionDenied => println!("Permission denied"),
            ErrorKind::AlreadyExists => println!("Port already open"),
            _ => println!("Other error: {}", e),
        },
    }

    Ok(())
}
