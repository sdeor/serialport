fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::ErrorKind;

    // Create a new serial port builder
    let builder = serialport::new("COM1", 115200);

    let mut port = builder.build()?;

    let bytes_to_read = port.bytes_to_read()?;
    println!("Bytes to read: {}", bytes_to_read);

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
