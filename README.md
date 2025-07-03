# Serial Port Communication Library

A cross-platform serial port communication library for Rust that provides fine-grained control over serial port configuration and lifecycle management.

## Why Another Serial Port Library?

Existing Rust serial port libraries have significant limitations that make them unsuitable for many real-world applications:

### 🚫 Problems with Existing Libraries

| Feature                     | `serialport`                  | `tokio-serial`                 | This Library                       |
| --------------------------- | ----------------------------- | ------------------------------ | ---------------------------------- |
| **Port Close/Reopen**       | ❌ No support                 | ❌ No support                  | ✅ Full lifecycle control          |
| **True Non-blocking I/O**   | ❌ `timeout=0` blocks forever | ❌ Inherited from `serialport` | ✅ `timeout=0` returns immediately |
| **Runtime Port Switching**  | ❌ Cannot change port path    | ❌ Cannot change port path     | ✅ Dynamic port switching          |
| **Proper Timeout Handling** | ❌ Broken on Windows          | ❌ Broken on Windows           | ✅ Correct timeout behavior        |

### 📋 Key Issues Addressed

1. **No Close/Reopen Support**: When working with devices that disconnect/reconnect (USB-to-serial adapters, removable devices), you need to close and reopen the port. Existing libraries don't support this properly.

2. **Broken Timeout Behavior**: Setting `timeout=0` should make reads non-blocking, but in `serialport` on Windows, it blocks forever instead of returning immediately when no data is available.

3. **No Runtime Port Switching**: Applications often need to switch between different serial ports at runtime, but existing libraries require creating entirely new instances.

4. **Limited Lifecycle Control**: No way to check if a port is open, or to manage the connection state properly.

## ✨ Features

-   **🔄 Full Lifecycle Control**: Open, close, and reopen serial ports as needed
-   **⚡ True Non-blocking I/O**: `timeout=0` actually works - returns immediately if no data available
-   **🎯 Type-safe Configuration**: Compile-time validation of serial port settings
-   **🔧 Runtime Reconfiguration**: Change port path, baud rate, and other settings on the fly
-   **🏗️ Builder Pattern**: Fluent API for easy port configuration
-   **🧵 Thread-safe**: Safe to use across multiple threads
-   **📚 Comprehensive Documentation**: Detailed examples and explanations

## 🚀 Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
serialport = { git = "https://github.com/sdeor/serialport" }
```

### Basic Usage

```rust
use std::io::{Read, Write};
use std::time::Duration;
use serialport::config::{DataBits, FlowControl, Parity, StopBits};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and configure a serial port
    let mut port = serialport::new("COM1", 9600)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .timeout(Duration::from_millis(1000)) // 1 second timeout
        .build()?;

    // Write data
    port.write_all(b"Hello, Serial!")?;

    // Read response
    let mut buffer = [0u8; 64];
    match port.read(&mut buffer) {
        Ok(bytes_read) => {
            println!("Received {} bytes: {:?}", bytes_read, &buffer[..bytes_read]);
        }
        Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
            println!("No data received within timeout");
        }
        Err(e) => return Err(e.into()),
    }

    // Close the port when done
    port.close()?;

    Ok(())
}
```

### Non-blocking I/O

```rust
use std::time::Duration;
use std::io::{Read, ErrorKind};

let mut port = serialport::new("COM1", 115200)
    .timeout(Duration::from_secs(0))  // Non-blocking
    .build()?;

let mut buffer = [0u8; 64];
match port.read(&mut buffer) {
    Ok(bytes_read) => println!("Got {} bytes immediately", bytes_read),
    Err(e) if e.kind() == ErrorKind::TimedOut => {
        println!("No data available right now");
        // This actually works, unlike other libraries!
    }
    Err(e) => eprintln!("Error: {}", e),
}

Ok::<(), std::io::Error>(())
```

### Port Lifecycle Management

```rust
let mut port = serialport::new("COM1", 9600).build()?;

// Check if port is open
if port.is_open() {
    println!("Port is ready");
}

// Close the port
port.close()?;
println!("Port closed");

// Reopen the same port
port.open()?;
println!("Port reopened");

// Switch to a different port
port.set_path("COM3".into())?;  // Automatically closes COM1 and opens COM3
println!("Now using COM3");

Ok::<(), std::io::Error>(())
```

### Runtime Configuration Changes

```rust
use std::time::Duration;
use serialport::config::{DataBits, FlowControl, Parity, StopBits};

let mut port = serialport::new("COM1", 9600).build()?;

// Change settings at runtime
port.set_baud_rate(115200)?;
port.set_data_bits(DataBits::Seven)?;
port.set_parity(Parity::Even)?;
port.set_stop_bits(StopBits::Two)?;
port.set_flow_control(FlowControl::Hardware)?;
port.set_timeout(Duration::from_millis(500))?;

println!("Configuration updated!");

Ok::<(), std::io::Error>(())
```

## 🛠️ Advanced Features

### Port Discovery

```rust
// List available ports
match serialport::available_ports() {
    Ok(ports) => {
        println!("Available ports:");
        for port in ports {
            println!("  {}", port.port_name);
        }
    }
    Err(e) => eprintln!("Failed to list ports: {}", e),
}
```

### Error Handling

The library provides detailed error information:

```rust
use std::io::ErrorKind;

let mut port = serialport::new("COM1", 9600).build()?;

match port.open() {
    Ok(()) => println!("Port opened successfully"),
    Err(e) => match e.kind() {
        ErrorKind::NotFound => println!("Port not found"),
        ErrorKind::PermissionDenied => println!("Permission denied"),
        ErrorKind::AlreadyExists => println!("Port already open"),
        _ => println!("Other error: {}", e),
    }
}

Ok::<(), std::io::Error>(())
```

## 🎯 Use Cases

This library is perfect for:

-   **Industrial automation** where devices may disconnect/reconnect
-   **Embedded development** requiring precise timing control
-   **Test equipment** that needs to switch between multiple serial devices
-   **Data acquisition** requiring non-blocking reads
-   **Protocol implementations** needing fine-grained control over serial parameters

## 🔧 Configuration Options

### Data Bits

-   `DataBits::Five` - 5 bits per character (rare)
-   `DataBits::Six` - 6 bits per character (uncommon)
-   `DataBits::Seven` - 7 bits per character (ASCII)
-   `DataBits::Eight` - 8 bits per character (most common)

### Parity

-   `Parity::None` - No parity checking
-   `Parity::Odd` - Odd parity
-   `Parity::Even` - Even parity
-   `Parity::Mark` - Mark parity (always 1)
-   `Parity::Space` - Space parity (always 0)

### Stop Bits

-   `StopBits::One` - 1 stop bit (most common)
-   `StopBits::OnePointFive` - 1.5 stop bits (rare)
-   `StopBits::Two` - 2 stop bits

### Flow Control

-   `FlowControl::None` - No flow control
-   `FlowControl::Software` - XON/XOFF flow control
-   `FlowControl::Hardware` - RTS/CTS flow control

## 🖥️ Platform Support

Currently supported platforms:

-   **Windows**: Full native support using WinAPI

**Coming soon:**

-   Linux (via termios)
-   macOS (via termios)
-   BSD variants

## 🆚 Comparison with Other Libraries

| Feature          | This Library     | `serialport` | `tokio-serial` |
| ---------------- | ---------------- | ------------ | -------------- |
| Close/Reopen     | ✅               | ❌           | ❌             |
| Non-blocking I/O | ✅               | ❌ (Windows) | ❌ (Windows)   |
| Port Switching   | ✅               | ❌           | ❌             |
| Type Safety      | ✅               | ✅           | ✅             |
| Cross Platform   | 🔄 (in progress) | ✅           | ✅             |
| Async Support    | 🔄 (planned)     | ❌           | ✅             |
| Documentation    | ✅               | ✅           | ✅             |

## 📞 Support

If you encounter any issues or have questions:

1. Search existing [issues](https://github.com/sdeor/serialport/issues) to see if your problem has already been reported
2. If not, please create a new issue with detailed information about your problem

## 🤝 Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for:

-   Additional platform support
-   Bug fixes
-   Feature enhancements
-   Documentation improvements

## 📄 License

This project is licensed under the MIT License.
