use std::io::Result;

pub trait Communication {
    /// Checks if the communication device is currently open.
    ///
    /// # Returns
    ///
    /// Returns `true` if the device is open and ready for I/O operations,
    /// `false` otherwise.
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
    /// if port.is_open() {
    ///     println!("Port is ready for communication");
    /// } else {
    ///     port.open()?;
    /// }
    /// # Ok::<(), std::io::Error>(())
    /// ```
    fn is_open(&self) -> bool;

    /// Opens the communication device for communication.
    ///
    /// This method must be called before any I/O operations can be performed.
    /// If the device is already open, this method returns an error.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the device was successfully opened, or an error
    /// if opening failed.
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
    /// let result = port.open();
    /// assert!(result.is_err());
    /// assert_eq!(
    ///     result.unwrap_err().kind(),
    ///     std::io::ErrorKind::AlreadyExists
    /// );
    ///
    /// # Ok::<(), std::io::Error>(())
    /// ```
    fn open(&mut self) -> Result<()>;

    /// Closes the communication device.
    ///
    /// After calling this method, no I/O operations can be performed until
    /// the device is reopened with `open()`. If the device is already closed,
    /// this method does nothing and returns success.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the device was successfully closed, or an error
    /// if closing failed.
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
    /// // Use the port...
    ///
    /// port.close()?;
    /// println!("Port closed successfully");
    /// # Ok::<(), std::io::Error>(())
    /// ```
    fn close(&mut self) -> Result<()>;
}
