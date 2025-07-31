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
//! use serialport::config::{DataBits, FlowControl, Parity, StopBits};
//!
//! // Create and configure a serial port
//! let mut port = serialport::new("COM1", 115200)
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

use std::{io, time::Duration};

pub mod communication;
pub mod config;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::ComPort;

use communication::Communication;
use config::{
    ClearBuffer, DataBits, FlowControl, Parity, SerialPortInfo, SerialPortType, StopBits,
    UsbPortInfo,
};

/// Builder for creating and configuring serial ports.
///
/// This struct uses the builder pattern to provide a fluent API for
/// configuring serial port parameters before creating the actual port.
/// All configuration methods return `self` to allow method chaining.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
/// use serialport::config::{DataBits, Parity, StopBits, FlowControl};
///
/// let port = serialport::new("COM1", 115200)
///     .data_bits(DataBits::Eight)
///     .parity(Parity::None)
///     .stop_bits(StopBits::One)
///     .flow_control(FlowControl::None)
///     .timeout(Duration::from_millis(1000))
///     .build()?;
/// # Ok::<(), std::io::Error>(())
/// ```
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerialPortBuilder {
    /// The port name, usually the device path
    path: String,
    /// The baud rate in symbols-per-second
    baud_rate: u32,
    /// Number of bits used to represent a character sent on the line
    data_bits: DataBits,
    /// The type of signalling to use for controlling data transfer
    flow_control: FlowControl,
    /// The type of parity to use for error checking
    parity: Parity,
    /// Number of bits to use to signal the end of a character
    stop_bits: StopBits,
    /// Amount of time to wait to receive data before timing out
    timeout: Duration,
}

impl SerialPortBuilder {
    /// Creates a new serial port builder with default settings.
    ///
    /// The default configuration is:
    /// - Port: Empty (must be set before building)
    /// - Baud rate: 9600
    /// - Data bits: 8
    /// - Flow control: None
    /// - Parity: None
    /// - Stop bits: 1
    /// - Timeout: 0 seconds (non-blocking)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::SerialPortBuilder;
    ///
    /// let builder = SerialPortBuilder::new();
    /// ```
    pub fn new() -> SerialPortBuilder {
        SerialPortBuilder {
            path: String::new(),
            baud_rate: 9600,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: Duration::ZERO,
        }
    }

    /// Sets the serial port device path.
    ///
    /// On Windows, this is typically a COM port like "COM1" or "COM2".
    /// On Unix-like systems, this is usually a device file like "/dev/ttyUSB0".
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the serial port device
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::SerialPortBuilder;
    ///
    /// let builder = SerialPortBuilder::new()
    ///     .path("COM1".into());  // Windows
    ///     
    /// let builder = SerialPortBuilder::new()
    ///     .path("/dev/ttyUSB0".into());  // Linux
    /// ```
    pub fn path<'a>(mut self, path: std::borrow::Cow<'a, str>) -> Self {
        self.path = path.into_owned();
        self
    }

    /// Sets the baud rate (speed) of the serial connection.
    ///
    /// The baud rate determines how many bits per second are transmitted.
    /// Both devices must use the same baud rate to communicate successfully.
    ///
    /// Common baud rates include: 9600, 19200, 38400, 57600, 115200.
    ///
    /// # Arguments
    ///
    /// * `baud_rate` - The baud rate in bits per second
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::SerialPortBuilder;
    ///
    /// let builder = SerialPortBuilder::new()
    ///     .baud_rate(115200);
    /// ```
    pub fn baud_rate(mut self, baud_rate: u32) -> Self {
        self.baud_rate = baud_rate;
        self
    }

    /// Sets the number of data bits per character.
    ///
    /// This determines how many bits are used to represent each character.
    /// Most modern applications use 8 data bits.
    ///
    /// # Arguments
    ///
    /// * `data_bits` - The number of data bits per character
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::{SerialPortBuilder, config::DataBits};
    ///
    /// let builder = SerialPortBuilder::new()
    ///     .data_bits(DataBits::Eight);
    /// ```
    pub fn data_bits(mut self, data_bits: DataBits) -> Self {
        self.data_bits = data_bits;
        self
    }

    /// Sets the flow control mechanism.
    ///
    /// Flow control prevents data loss by managing when data should be
    /// transmitted based on the receiver's ability to process it.
    ///
    /// # Arguments
    ///
    /// * `flow_control` - The flow control method to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::{SerialPortBuilder, config::FlowControl};
    ///
    /// let builder = SerialPortBuilder::new()
    ///     .flow_control(FlowControl::Hardware);
    /// ```
    pub fn flow_control(mut self, flow_control: FlowControl) -> Self {
        self.flow_control = flow_control;
        self
    }

    /// Sets the parity bit configuration for error detection.
    ///
    /// Parity bits can detect single-bit transmission errors.
    /// Both devices must use the same parity setting.
    ///
    /// # Arguments
    ///
    /// * `parity` - The parity configuration to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::{SerialPortBuilder, config::Parity};
    ///
    /// let builder = SerialPortBuilder::new()
    ///     .parity(Parity::Even);
    /// ```
    pub fn parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }

    /// Sets the number of stop bits.
    ///
    /// Stop bits provide a pause between characters to allow the receiver
    /// to prepare for the next character.
    ///
    /// # Arguments
    ///
    /// * `stop_bits` - The number of stop bits to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::{SerialPortBuilder, config::StopBits};
    ///
    /// let builder = SerialPortBuilder::new()
    ///     .stop_bits(StopBits::Two);
    /// ```
    pub fn stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = stop_bits;
        self
    }

    /// Sets the read timeout duration.
    ///
    /// This determines how long read operations will wait for data before
    /// timing out. A timeout of zero means non-blocking operation - reads
    /// will return immediately if no data is available.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The timeout duration for read operations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use serialport::SerialPortBuilder;
    ///
    /// let builder = SerialPortBuilder::new()
    ///     .timeout(Duration::from_millis(500));  // 500ms timeout
    ///     
    /// let builder = SerialPortBuilder::new()
    ///     .timeout(Duration::from_secs(0));      // Non-blocking
    /// ```
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Creates a new serial port with the configured settings.
    ///
    /// This method consumes the builder and creates a new `SerialPort` instance.
    /// If the port path is not set, the serial port is created but not
    /// automatically opened - you must call `set_port()` and `open()` or ensure
    /// a port path is set (which will auto-open the port).
    ///
    /// # Returns
    ///
    /// Returns a `Result<SerialPort, std::io::Error>` containing either the
    /// configured serial port or an error if creation failed.
    ///
    /// # Errors
    ///
    /// This method can fail if:
    /// - The specified port path is invalid
    /// - The port is already in use by another process
    /// - The system lacks permission to access the port
    /// - The port hardware is not available
    ///
    /// # Examples
    ///
    /// ```rust
    /// let port = serialport::new("COM1", 115200)
    ///     .build()?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    #[must_use]
    pub fn build(self) -> io::Result<Box<dyn SerialPort>> {
        #[cfg(windows)]
        return ComPort::new(self).map(|port| Box::new(port) as Box<dyn SerialPort>);

        // Placeholder for non-Windows implementation
        #[cfg(not(windows))]
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Serial port builder is not implemented for this platform",
        ))
    }
}

impl Default for SerialPortBuilder {
    /// Creates a new serial port builder with default settings.
    ///
    /// This is equivalent to calling `SerialPortBuilder::new()`.
    fn default() -> Self {
        Self::new()
    }
}

mod private {
    pub trait Private {
        /// Sets the raw path of the serial port.
        /// This method is used internally to change the port path
        /// and must be implemented by all serial port types.
        ///
        /// # Arguments
        ///
        /// * `path` - The new port path
        ///
        /// # Returns
        ///
        /// Returns `Ok(())` if the port path was successfully changed,
        /// or an error if the operation failed.
        ///
        /// # Safety
        ///
        /// This method is unsafe because it allows changing the port path
        /// without checking if the port is currently open.
        fn set_raw_path<'a>(&mut self, path: std::borrow::Cow<'a, str>) -> std::io::Result<()>;
    }
}

/// The `SerialPort` trait defines a common interface for serial port communication,
/// providing methods for opening, closing, configuring, and querying the state of a serial port.
///
/// Types implementing this trait must support both reading and writing, as well as thread-safe access.
///
/// # Overview
///
/// This trait abstracts over platform-specific serial port implementations, allowing for
/// portable and flexible serial communication. It provides methods to:
///
/// - Open and close the port
/// - Query and change port settings (baud rate, data bits, parity, stop bits, flow control, timeout)
/// - Get the port path/name
/// - Query the number of bytes available for reading or writing
/// - Clear port buffers
/// - Clone the port handle
///
/// # Example
///
/// ```rust
/// use std::time::Duration;
/// use serialport::{SerialPortBuilder, config::DataBits};
///
/// let mut port = SerialPortBuilder::new()
///     .path("COM1".into())
///     .baud_rate(9600)
///     .data_bits(DataBits::Eight)
///     .timeout(Duration::from_secs(1))
///     .build()?;
///
/// if !port.is_open() {
///     port.open()?;
/// }
///
/// port.write_all(b"Hello, serial port!")?;
/// let mut buffer = [0u8; 128];
/// // This read will generate an error since we are not sending any data during the test
/// // and the timeout is set to 1000ms.
/// // In a real application, you would expect data to be available.
/// let bytes_read_result = port.read(&mut buffer);
///
/// assert!(bytes_read_result.is_err());
/// assert_eq!(
///     bytes_read_result.unwrap_err().kind(),
///     std::io::ErrorKind::TimedOut
/// );
///
/// port.close()?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Thread Safety
///
/// Implementors must ensure that all methods are safe to call from multiple threads,
/// as the trait requires `Send`.
///
/// # Errors
///
/// Most methods return `std::io::Result` to indicate I/O errors, permission issues,
/// or invalid configuration.
///
/// # Implementors
///
/// Implement this trait for platform-specific serial port types to provide
/// a unified API for serial communication.
pub trait SerialPort: Send + Communication + io::Read + io::Write + private::Private {
    /// Creates a clone of this serial port handle.
    ///
    /// This allows you to read and write simultaneously from the same serial
    /// connection using multiple handles. Each handle maintains its own state
    /// but shares the underlying connection.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Box<dyn SerialPort>, std::io::Error>` containing either
    /// a new handle to the same port or an error if cloning failed.
    ///
    /// # Errors
    ///
    /// This function returns an error if the serial port couldn't be cloned,
    /// which may occur if the underlying system doesn't support handle duplication
    /// or if there are insufficient system resources.
    ///
    /// # Important Notes
    ///
    /// - For true asynchronous serial port operations, consider using libraries
    ///   like `mio-serial` or `tokio-serial` instead
    /// - Be careful when changing settings from multiple cloned handles, as
    ///   settings are cached per object and conflicting changes can cause
    ///   unexpected behavior
    /// - Both handles can be used independently and are safe to use from
    ///   different threads
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::SerialPortBuilder;
    ///
    /// let port1 = SerialPortBuilder::new()
    ///     .path("COM1".into())
    ///     .build()?;
    ///
    /// let mut port2 = port1.try_clone()?;
    ///
    /// // Use port1 for reading and port2 for writing
    /// // (in separate threads if needed)
    /// # Ok::<(), std::io::Error>(())
    /// ```
    fn try_clone(&self) -> io::Result<Box<dyn SerialPort>>;

    /// Gets the current port path/name.
    ///
    /// # Returns
    ///
    /// Returns the path or name of the serial port device.
    /// This name may not be the canonical device name and instead be shorthand.
    /// Additionally it may not exist for virtual ports.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::SerialPortBuilder;
    ///
    /// let port = SerialPortBuilder::new()
    ///     .path("COM1".into())
    ///     .build()?;
    ///
    /// println!("Using port: {}", port.path().unwrap_or("Unknown".to_string()));
    /// # Ok::<(), std::io::Error>(())
    /// ```
    fn path(&self) -> Option<String>;

    /// Gets the current baud rate.
    ///
    /// # Returns
    ///
    /// Returns a `Result<u32, std::io::Error>` containing either the
    /// current baud rate or an error if retrieval failed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::SerialPortBuilder;
    ///
    /// let port = SerialPortBuilder::new()
    ///     .path("COM1".into())
    ///     .baud_rate(115200)
    ///     .build()?;
    ///
    /// println!("Baud rate: {}", port.baud_rate()?);
    /// # Ok::<(), std::io::Error>(())
    /// ```
    fn baud_rate(&self) -> io::Result<u32>;

    /// Gets the current data bits setting.
    ///
    /// # Returns
    ///
    /// Returns a `Result<DataBits, std::io::Error>` containing either
    /// the current data bits setting or an error if retrieval failed.
    fn data_bits(&self) -> io::Result<DataBits>;

    /// Gets the current flow control setting.
    ///
    /// # Returns
    ///
    /// Returns a `Result<FlowControl, std::io::Error>` containing either
    /// the current flow control setting or an error if retrieval failed.
    fn flow_control(&self) -> io::Result<FlowControl>;

    /// Gets the current parity setting.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Parity, std::io::Error>` containing either
    /// the current parity setting or an error if retrieval failed.
    fn parity(&self) -> io::Result<Parity>;

    /// Gets the current stop bits setting.
    ///
    /// # Returns
    ///
    /// Returns a `Result<StopBits, std::io::Error>` containing either
    /// the current stop bits setting or an error if retrieval failed.
    fn stop_bits(&self) -> io::Result<StopBits>;

    /// Gets the current timeout setting.
    ///
    /// # Returns
    ///
    /// Returns the current timeout duration for read operations.
    fn timeout(&self) -> Duration;

    /// Gets the number of bytes available to be read from the input buffer.
    ///
    /// This function returns the number of bytes that have been received
    /// and are waiting in the input buffer to be read. This can be useful
    /// for determining if data is available before attempting a read operation.
    ///
    /// # Returns
    ///
    /// Returns a `Result<u32, std::io::Error>` containing either the number
    /// of bytes available for reading or an error if the operation failed.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    /// - `NoDevice` if the device was disconnected
    /// - `Io` for any other type of I/O error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::SerialPortBuilder;
    ///
    /// let port = SerialPortBuilder::new()
    ///     .path("COM1".into())
    ///     .build()?;
    ///
    /// let available = port.bytes_to_read()?;
    /// println!("Bytes available for reading: {}", available);
    /// # Ok::<(), std::io::Error>(())
    /// ```
    fn bytes_to_read(&self) -> io::Result<u32>;

    /// Gets the number of bytes written to the output buffer awaiting transmission.
    ///
    /// This function returns the number of bytes that have been written to
    /// the output buffer but have not yet been transmitted. This can be useful
    /// for flow control or determining when all data has been sent.
    ///
    /// # Returns
    ///
    /// Returns a `Result<u32, std::io::Error>` containing either the number
    /// of bytes waiting to be transmitted or an error if the operation failed.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    /// - `NoDevice` if the device was disconnected
    /// - `Io` for any other type of I/O error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::SerialPortBuilder;
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .path("COM1".into())
    ///     .build()?;
    ///
    /// port.write_all(b"Hello, world!")?;
    /// let pending = port.bytes_to_write()?;
    /// println!("Bytes pending transmission: {}", pending);
    /// # Ok::<(), std::io::Error>(())
    /// ```
    fn bytes_to_write(&self) -> io::Result<u32>;

    /// Changes the port path and reopens the connection if necessary.
    ///
    /// If the port is currently open, it will be closed, the path will be
    /// changed, and then the port will be reopened with the new path.
    ///
    /// # Arguments
    ///
    /// * `path` - The new port path
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the port path was successfully changed,
    /// or an error if the operation failed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::Error;
    /// use serialport::SerialPortBuilder;
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .path("COM1".into())
    ///     .build()?;
    ///
    /// // Change the port path
    /// if let Err(e) = port.set_path("COM2".into()) {
    ///     eprintln!("Failed to change port: {}", e);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    fn set_path<'a>(&mut self, path: std::borrow::Cow<'a, str>) -> io::Result<()> {
        let was_open = self.is_open();
        if was_open {
            self.close()?;
        }

        self.set_raw_path(path)?;

        if was_open {
            self.open()?;
        }

        Ok(())
    }

    /// Sets the baud rate for the serial port.
    ///
    /// # Arguments
    ///
    /// * `baud_rate` - The desired baud rate (e.g., 9600, 115200)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the baud rate was successfully set,
    /// or an error if the operation failed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::Error;
    ///
    /// let mut port = serialport::new("COM1", 9600)
    ///     .build()?;
    ///
    /// // Set a new baud rate
    /// if let Err(e) = port.set_baud_rate(115200) {
    ///     eprintln!("Failed to set baud rate: {}", e);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    fn set_baud_rate(&mut self, baud_rate: u32) -> io::Result<()>;

    /// Sets the number of data bits for the serial port.
    ///
    /// # Arguments
    ///
    /// * `data_bits` - The desired number of data bits (5, 6, 7, or 8)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the data bits were successfully set,
    /// or an error if the operation failed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::Error;
    /// use serialport::{SerialPortBuilder, config::DataBits};
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .path("COM1".into())
    ///     .build()?;
    ///
    /// // Set to 8 data bits (most common)
    /// if let Err(e) = port.set_data_bits(DataBits::Eight) {
    ///     eprintln!("Failed to set data bits: {}", e);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    fn set_data_bits(&mut self, data_bits: DataBits) -> io::Result<()>;

    /// Sets the flow control for the serial port.
    ///
    /// Flow control manages the flow of data between devices to prevent
    /// buffer overflow and ensure reliable communication.
    ///
    /// # Arguments
    ///
    /// * `flow_control` - The desired flow control setting
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the flow control was successfully set,
    /// or an error if the operation failed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::Error;
    /// use serialport::{SerialPortBuilder, config::FlowControl};
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .path("COM1".into())
    ///     .build()?;
    ///
    /// // Disable flow control (most common for simple applications)
    /// if let Err(e) = port.set_flow_control(FlowControl::None) {
    ///     eprintln!("Failed to set flow control: {}", e);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    fn set_flow_control(&mut self, flow_control: FlowControl) -> io::Result<()>;

    /// Sets the parity checking for the serial port.
    ///
    /// Parity is an error-checking mechanism that can detect some
    /// transmission errors in serial communication.
    ///
    /// # Arguments
    ///
    /// * `parity` - The desired parity setting
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the parity was successfully set,
    /// or an error if the operation failed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::Error;
    /// use serialport::{SerialPortBuilder, config::Parity};
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .path("COM1".into())
    ///     .build()?;
    ///
    /// // Set no parity (most common)
    /// if let Err(e) = port.set_parity(Parity::None) {
    ///     eprintln!("Failed to set parity: {}", e);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    fn set_parity(&mut self, parity: Parity) -> io::Result<()>;

    /// Sets the number of stop bits for the serial port.
    ///
    /// Stop bits indicate the end of a data frame in serial communication.
    /// Most applications use one stop bit, but some may require two for
    /// slower or more error-prone connections.
    ///
    /// # Arguments
    ///
    /// * `stop_bits` - The desired number of stop bits
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the stop bits were successfully set,
    /// or an error if the operation failed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::Error;
    /// use serialport::{SerialPortBuilder, config::StopBits};
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .path("COM1".into())
    ///     .build()?;
    ///
    /// // Set one stop bit (most common)
    /// if let Err(e) = port.set_stop_bits(StopBits::One) {
    ///     eprintln!("Failed to set stop bits: {}", e);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    fn set_stop_bits(&mut self, stop_bits: StopBits) -> io::Result<()>;

    /// Sets the timeout for read operations.
    ///
    /// This timeout controls how long I/O operations will wait before timing out.
    /// Read operations will wait this duration for incoming data, and write
    /// operations will wait this duration to send data. If the timeout is zero,
    /// operations return immediately without blocking - reads return an error
    /// if no data is available, and writes return an error if the operation
    /// cannot complete instantly.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The desired timeout duration
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the timeout was successfully set,
    /// or an error if the operation failed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use std::io::Error;
    /// use serialport::SerialPortBuilder;
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .path("COM1".into())
    ///     .build()?;
    ///
    /// // Set a 5-second timeout
    /// if let Err(e) = port.set_timeout(Duration::from_secs(5)) {
    ///     eprintln!("Failed to set timeout: {}", e);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    fn set_timeout(&mut self, timeout: std::time::Duration) -> io::Result<()>;

    /// Clears the specified input or output buffer.
    ///
    /// This function discards any data in the specified buffer(s), which can
    /// be useful for clearing stale data before beginning a new communication
    /// session or recovering from communication errors.
    ///
    /// # Arguments
    ///
    /// * `buffer_to_clear` - Specifies which buffer(s) to clear (input, output, or both)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the buffer was successfully cleared, or an error
    /// if the operation failed.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    /// - `NoDevice` if the device was disconnected
    /// - `Io` for any other type of I/O error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::{SerialPortBuilder, config::ClearBuffer};
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .path("COM1".into())
    ///     .build()?;
    ///
    /// // Clear input buffer to discard any stale received data
    /// port.clear(ClearBuffer::Input)?;
    ///
    /// // Clear both input and output buffers
    /// port.clear(ClearBuffer::All)?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    fn clear(&self, buffer_to_clear: ClearBuffer) -> io::Result<()>;
}

/// Construct a builder of `SerialPort` objects
///
/// `SerialPort` objects are built using the Builder pattern through the `new` function. The
/// resultant `SerialPortBuilder` object can be copied, reconfigured, and saved making working with
/// multiple serial ports a little easier.
///
/// To open a new serial port:
/// ```rust
/// serialport::new("COM1", 9600).build().expect("Failed to open port");
/// ```
pub fn new<'a>(path: impl Into<std::borrow::Cow<'a, str>>, baud_rate: u32) -> SerialPortBuilder {
    SerialPortBuilder {
        path: path.into().into_owned(),
        baud_rate,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::ZERO,
    }
}

/// Returns a list of all serial ports on the system.
///
/// This method scans the system for available serial port devices
/// and returns their names/paths.
///
/// # Returns
///
/// Returns a `Result<Vec<String>, std::io::Error>` containing either
/// a list of available port names or an error if the scan failed.
///
/// # Examples
///
/// ```rust
/// match serialport::available_ports() {
///     Ok(ports) => {
///         println!("Available ports:");
///         for port in ports {
///             println!("  {} ({})", port.port_name, {
///                 match port.port_type {
///                     serialport::config::SerialPortType::UsbPort(info) => {
///                         format!("USB: {} {}", info.vid, info.pid)
///                     }
///                     serialport::config::SerialPortType::BluetoothPort => {
///                         "Bluetooth".to_string()
///                     }
///                     serialport::config::SerialPortType::PciPort => "PCI".to_string(),
///                     serialport::config::SerialPortType::Unknown => "Unknown".to_string(),
///                 }
///             });
///         }
///     }
///     Err(e) => eprintln!("Failed to list ports: {}", e),
/// }
/// ```
pub fn available_ports() -> io::Result<Vec<SerialPortInfo>> {
    #[cfg(unix)]
    return crate::posix::available_ports();

    #[cfg(windows)]
    return crate::windows::available_ports();

    #[cfg(not(any(unix, windows)))]
    Err(Error::new(
        ErrorKind::Unknown,
        "available_ports() not implemented for platform",
    ))
}

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct README;
