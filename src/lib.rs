//! # Serial Port Communication Library
//!
//! A cross-platform serial port communication library for Rust that provides fine-grained
//! control over serial port configuration and lifecycle management.
//!
//! ## Features
//!
//! - **Full lifecycle control**: Open, close, and reopen serial ports as needed
//! - **Proper timeout handling**: True non-blocking I/O with timeout=0 support
//! - **Type-safe configuration**: Compile-time validation of serial port settings
//! - **Builder pattern**: Fluent API for easy port configuration
//! - **Thread-safe**: Safe to use across multiple threads
//!
//! ## Quick Start
//!
//! ```rust
//! use std::time::Duration;
//! use std::io::{Read, Write};
//! use serialport::{builder::SerialPortBuilder, config::{DataBits, FlowControl, Parity, StopBits}};
//!
//! // Create and configure a serial port
//! let mut port = SerialPortBuilder::new()
//!     .port("COM1")
//!     .baud_rate(115200)
//!     .data_bits(DataBits::Eight)
//!     .parity(Parity::None)
//!     .stop_bits(StopBits::One)
//!     .flow_control(FlowControl::None)
//!     .timeout(Duration::from_millis(1000))
//!     .build()?;
//!
//! // Write data
//! port.write_all(b"Hello, Serial!")?;
//!
//! // Read data
//! let mut buffer = [0u8; 64];
//! // This read will generate an error since we are not sending any data during the test
//! // and the timeout is set to 1000ms.
//! // In a real application, you would expect data to be available.
//! let bytes_read_result = port.read(&mut buffer);
//!
//! assert!(bytes_read_result.is_err());
//! assert_eq!(
//!     bytes_read_result.unwrap_err().kind(),
//!     std::io::ErrorKind::TimedOut
//! );
//!
//! // Close the port when done
//! port.close()?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! ## Platform Support
//!
//! Currently supported platforms:
//! - **Windows**: Full native support using WinAPI
//!
//! ## Why This Library?
//!
//! Existing serial port libraries have significant limitations:
//! - No proper close/reopen support for device lifecycle management
//! - Broken timeout behavior (timeout=0 blocks instead of returning immediately)
//! - Limited control over port configuration and state
//!
//! This library addresses these issues by providing direct platform integration
//! and proper state management.

pub mod builder;
pub mod config;
pub mod serialport;

pub use builder::SerialPortBuilder;
pub use serialport::SerialPort;
