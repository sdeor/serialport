use std::{ffi::OsString, path::Path, time::Duration};

use crate::{
    SerialPort,
    config::{DataBits, FlowControl, Parity, StopBits},
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
/// use serialport::{builder::SerialPortBuilder, config::{DataBits, Parity, StopBits, FlowControl}};
///
/// let port = SerialPortBuilder::new()
///     .port("COM1") // Windows example
///     .baud_rate(115200)
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
    pub(super) port: OsString,
    /// The baud rate in symbols-per-second
    pub(super) baud_rate: u32,
    /// Number of bits used to represent a character sent on the line
    pub(super) data_bits: DataBits,
    /// The type of signalling to use for controlling data transfer
    pub(super) flow_control: FlowControl,
    /// The type of parity to use for error checking
    pub(super) parity: Parity,
    /// Number of bits to use to signal the end of a character
    pub(super) stop_bits: StopBits,
    /// Amount of time to wait to receive data before timing out
    pub(super) timeout: Duration,
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
            port: OsString::new(),
            baud_rate: 9600,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: Duration::from_secs(0),
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
    ///     .port("COM1");  // Windows
    ///     
    /// let builder = SerialPortBuilder::new()
    ///     .port("/dev/ttyUSB0");  // Linux
    /// ```
    pub fn port<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.port = path.as_ref().as_os_str().into();
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
    /// use serialport::{builder::SerialPortBuilder, config::DataBits};
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
    /// use serialport::{builder::SerialPortBuilder, config::FlowControl};
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
    /// use serialport::{builder::SerialPortBuilder, config::Parity};
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
    /// use serialport::{builder::SerialPortBuilder, config::StopBits};
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
    /// use serialport::SerialPortBuilder;
    ///
    /// let port = SerialPortBuilder::new()
    ///     .port("COM1")
    ///     .baud_rate(115200)
    ///     .build()?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    #[must_use]
    pub fn build(&self) -> std::io::Result<SerialPort> {
        SerialPort::new(&self)
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
