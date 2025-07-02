//! Cross-platform serial port implementation.
//!
//! This module provides the main `SerialPort` struct that abstracts over
//! platform-specific serial port implementations while providing a unified API.

use crate::SerialPortBuilder;
use crate::config::{DataBits, FlowControl, Parity, StopBits};

#[cfg(windows)]
mod windows;

/// A cross-platform serial port implementation.
///
/// This struct provides a unified interface for serial port communication
/// across different platforms. It supports full lifecycle management including
/// opening, closing, and reopening ports, as well as runtime configuration
/// changes.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
/// use std::io::{Read, Write};
/// use serialport::{builder::SerialPortBuilder, config::{DataBits, FlowControl, Parity, StopBits}};
///
/// // Create a serial port
/// let mut port = SerialPortBuilder::new()
///     .port("COM1")
///     .baud_rate(9600)
///     .timeout(Duration::from_millis(1000))
///     .build()?;
///
/// // Write data
/// port.write_all(b"Hello")?;
///
/// // Read response
/// let mut buffer = [0u8; 64];
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
/// // Close when done
/// port.close()?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct SerialPort {
    #[cfg(windows)]
    inner: windows::SerialPort,
}

impl SerialPort {
    /// Returns a list of available serial ports on the system.
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
    /// use serialport::SerialPort;
    ///
    /// match SerialPort::available_ports() {
    ///     Ok(ports) => {
    ///         println!("Available ports:");
    ///         for port in ports {
    ///             println!("  {}", port);
    ///         }
    ///     }
    ///     Err(e) => eprintln!("Failed to list ports: {}", e),
    /// }
    /// ```
    #[cfg(windows)]
    pub fn available_ports() -> std::io::Result<Vec<String>> {
        windows::SerialPort::available_ports()
    }

    /// Creates a new serial port from a builder configuration.
    ///
    /// This is typically called by `SerialPortBuilder::build()` rather than
    /// directly by user code.
    ///
    /// # Arguments
    ///
    /// * `builder` - The builder containing the port configuration
    ///
    /// # Returns
    ///
    /// Returns a `Result<SerialPort, std::io::Error>` containing either
    /// the configured port or an error if creation failed.
    #[cfg(windows)]
    pub(super) fn new(builder: &SerialPortBuilder) -> std::io::Result<Self> {
        Ok(Self {
            inner: windows::SerialPort::new(builder)?,
        })
    }

    /// Creates a clone of this serial port.
    ///
    /// This creates a new handle to the same underlying serial port device.
    /// Both handles can be used independently and are safe to use from
    /// different threads.
    ///
    /// # Returns
    ///
    /// Returns a `Result<SerialPort, std::io::Error>` containing either
    /// a new handle to the same port or an error if cloning failed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::SerialPortBuilder;
    ///
    /// let port1 = SerialPortBuilder::new()
    ///     .port("COM1")
    ///     .build()?;
    ///
    /// let port2 = port1.try_clone()?;
    /// // Both port1 and port2 can now be used independently
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn try_clone(&self) -> std::io::Result<Self> {
        Ok(Self {
            inner: self.inner.try_clone()?,
        })
    }

    /// Checks if the serial port is currently open.
    ///
    /// # Returns
    ///
    /// Returns `true` if the port is open and ready for I/O operations,
    /// `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::SerialPortBuilder;
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .port("COM1")
    ///     .build()?;
    ///
    /// if port.is_open() {
    ///     println!("Port is ready for communication");
    /// } else {
    ///     port.open()?;
    /// }
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn is_open(&self) -> bool {
        self.inner.is_open()
    }

    /// Opens the serial port for communication.
    ///
    /// This method must be called before any I/O operations can be performed.
    /// If the port is already open, this method returns an error.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the port was successfully opened, or an error
    /// if opening failed.
    ///
    /// # Errors
    ///
    /// This method can fail if:
    /// - The port is already open
    /// - The port path is invalid or empty
    /// - The port is in use by another process
    /// - Insufficient permissions to access the port
    /// - Hardware is not available
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::SerialPortBuilder;
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .port("COM1")
    ///     .build()?;
    ///
    /// let result = port.open();
    /// assert!(result.is_err());
    /// assert_eq!(
    ///     result.unwrap_err().kind(),
    ///     std::io::ErrorKind::AlreadyExists
    /// );
    ///
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn open(&mut self) -> std::io::Result<()> {
        self.inner.open()
    }

    /// Closes the serial port.
    ///
    /// After calling this method, no I/O operations can be performed until
    /// the port is reopened with `open()`. If the port is already closed,
    /// this method does nothing and returns success.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the port was successfully closed, or an error
    /// if closing failed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::SerialPortBuilder;
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .port("COM1")
    ///     .build()?;
    ///
    /// // Use the port...
    ///
    /// port.close()?;
    /// println!("Port closed successfully");
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn close(&mut self) -> std::io::Result<()> {
        self.inner.close()
    }

    /// Gets the current port path/name.
    ///
    /// # Returns
    ///
    /// Returns the path or name of the serial port device.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialport::SerialPortBuilder;
    ///
    /// let port = SerialPortBuilder::new()
    ///     .port("COM1")
    ///     .build()?;
    ///
    /// println!("Using port: {}", port.port());
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn port(&self) -> &str {
        self.inner.port()
    }

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
    ///     .port("COM1")
    ///     .baud_rate(115200)
    ///     .build()?;
    ///
    /// println!("Baud rate: {}", port.baud_rate()?);
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn baud_rate(&self) -> std::io::Result<u32> {
        self.inner.baud_rate()
    }

    /// Gets the current data bits setting.
    ///
    /// # Returns
    ///
    /// Returns a `Result<DataBits, std::io::Error>` containing either
    /// the current data bits setting or an error if retrieval failed.
    pub fn data_bits(&self) -> std::io::Result<DataBits> {
        self.inner.data_bits()
    }

    /// Gets the current flow control setting.
    ///
    /// # Returns
    ///
    /// Returns a `Result<FlowControl, std::io::Error>` containing either
    /// the current flow control setting or an error if retrieval failed.
    pub fn flow_control(&self) -> std::io::Result<FlowControl> {
        self.inner.flow_control()
    }

    /// Gets the current parity setting.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Parity, std::io::Error>` containing either
    /// the current parity setting or an error if retrieval failed.
    pub fn parity(&self) -> std::io::Result<Parity> {
        self.inner.parity()
    }

    /// Gets the current stop bits setting.
    ///
    /// # Returns
    ///
    /// Returns a `Result<StopBits, std::io::Error>` containing either
    /// the current stop bits setting or an error if retrieval failed.
    pub fn stop_bits(&self) -> std::io::Result<StopBits> {
        self.inner.stop_bits()
    }

    /// Gets the current timeout setting.
    ///
    /// # Returns
    ///
    /// Returns the current timeout duration for read operations.
    pub fn timeout(&self) -> std::time::Duration {
        self.inner.timeout()
    }

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
    /// use serialport::builder::SerialPortBuilder;
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .port("COM1")
    ///     .build()?;
    ///
    /// // Change the port path
    /// if let Err(e) = port.set_port("COM2") {
    ///     eprintln!("Failed to change port: {}", e);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    pub fn set_port<P: AsRef<std::path::Path>>(&mut self, path: P) -> std::io::Result<()> {
        let was_open = self.is_open();
        if was_open {
            self.close()?;
        }

        self.inner.set_port(path);

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
    /// use serialport::{builder::SerialPortBuilder, config::Parity};
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .port("COM1")
    ///     .baud_rate(9600)
    ///     .build()?;
    ///
    /// // Set a new baud rate
    /// if let Err(e) = port.set_baud_rate(115200) {
    ///     eprintln!("Failed to set baud rate: {}", e);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    pub fn set_baud_rate(&mut self, baud_rate: u32) -> std::io::Result<()> {
        self.inner.set_baud_rate(baud_rate)
    }

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
    /// use serialport::{builder::SerialPortBuilder, config::DataBits};
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .port("COM1")
    ///     .build()?;
    ///
    /// // Set to 8 data bits (most common)
    /// if let Err(e) = port.set_data_bits(DataBits::Eight) {
    ///     eprintln!("Failed to set data bits: {}", e);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    pub fn set_data_bits(&mut self, data_bits: DataBits) -> std::io::Result<()> {
        self.inner.set_data_bits(data_bits)
    }

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
    /// use serialport::{builder::SerialPortBuilder, config::FlowControl};
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .port("COM1")
    ///     .build()?;
    ///
    /// // Disable flow control (most common for simple applications)
    /// if let Err(e) = port.set_flow_control(FlowControl::None) {
    ///     eprintln!("Failed to set flow control: {}", e);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    pub fn set_flow_control(&mut self, flow_control: FlowControl) -> std::io::Result<()> {
        self.inner.set_flow_control(flow_control)
    }

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
    /// use serialport::{builder::SerialPortBuilder, config::Parity};
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .port("COM1")
    ///     .build()?;
    ///
    /// // Set no parity (most common)
    /// if let Err(e) = port.set_parity(Parity::None) {
    ///     eprintln!("Failed to set parity: {}", e);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    pub fn set_parity(&mut self, parity: Parity) -> std::io::Result<()> {
        self.inner.set_parity(parity)
    }

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
    /// use serialport::{builder::SerialPortBuilder, config::StopBits};
    ///
    /// let mut port = SerialPortBuilder::new()
    ///     .port("COM1")
    ///     .build()?;
    ///
    /// // Set one stop bit (most common)
    /// if let Err(e) = port.set_stop_bits(StopBits::One) {
    ///     eprintln!("Failed to set stop bits: {}", e);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    pub fn set_stop_bits(&mut self, stop_bits: StopBits) -> std::io::Result<()> {
        self.inner.set_stop_bits(stop_bits)
    }

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
    ///     .port("COM1")
    ///     .build()?;
    ///
    /// // Set a 5-second timeout
    /// if let Err(e) = port.set_timeout(Duration::from_secs(5)) {
    ///     eprintln!("Failed to set timeout: {}", e);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    pub fn set_timeout(&mut self, timeout: std::time::Duration) -> std::io::Result<()> {
        self.inner.set_timeout(timeout)
    }
}

impl std::io::Read for SerialPort {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl std::io::Write for SerialPort {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
