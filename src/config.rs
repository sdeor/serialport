//! Serial port configuration and builder pattern implementation.
//!
//! This module provides type-safe enums for serial port configuration and a builder
//! pattern for creating serial port instances with specific settings.

use std::fmt;

/// Number of data bits per character in serial communication.
///
/// This determines how many bits are used to represent each character
/// transmitted over the serial line.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DataBits {
    /// 5 bits per character
    ///
    /// Rarely used in modern applications. Historically used with
    /// Baudot code and some older teletype systems.
    Five = 5,

    /// 6 bits per character
    ///
    /// Uncommon in modern usage. Sometimes used in specialized
    /// embedded systems or legacy protocols.
    Six,

    /// 7 bits per character
    ///
    /// Commonly used for ASCII text transmission. Each character
    /// uses 7 bits, allowing for 128 different characters.
    Seven,

    /// 8 bits per character
    ///
    /// Most common setting for modern serial communication.
    /// Allows for full byte transmission and extended ASCII.
    Eight,
}

impl fmt::Display for DataBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            DataBits::Five => write!(f, "5"),
            DataBits::Six => write!(f, "6"),
            DataBits::Seven => write!(f, "7"),
            DataBits::Eight => write!(f, "8"),
        }
    }
}

/// Flow control mechanism for managing data transmission.
///
/// Flow control prevents data loss by controlling when the sender
/// should pause or resume transmission based on the receiver's ability
/// to process incoming data.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FlowControl {
    /// No flow control
    ///
    /// The sender transmits data continuously without checking
    /// if the receiver is ready. This is the simplest but least
    /// reliable option for high-speed or high-volume data.
    None,

    /// Software flow control using XON/XOFF bytes
    ///
    /// Uses special control characters (XON=0x11, XOFF=0x13) to
    /// signal when to pause/resume transmission. The receiver sends
    /// XOFF to pause the sender and XON to resume.
    Software,

    /// Hardware flow control using RTS/CTS signals
    ///
    /// Uses dedicated hardware lines (Request To Send/Clear To Send)
    /// to control data flow. More reliable than software flow control
    /// as it doesn't depend on the data stream.
    Hardware,
}

impl fmt::Display for FlowControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            FlowControl::None => write!(f, "None"),
            FlowControl::Software => write!(f, "XON/XOFF"),
            FlowControl::Hardware => write!(f, "RTS/CTS"),
        }
    }
}

/// Parity bit configuration for error detection.
///
/// Parity bits are used to detect single-bit errors in transmission.
/// The parity bit is set based on the number of 1 bits in the data.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Parity {
    /// No parity bit
    ///
    /// No error detection is performed. This provides the maximum
    /// data throughput but no error detection capability.
    None,

    /// Odd parity
    ///
    /// The parity bit is set so that the total number of 1 bits
    /// (including the parity bit) is odd. If the count is already
    /// odd, the parity bit is 0; if even, the parity bit is 1.
    Odd,

    /// Even parity
    ///
    /// The parity bit is set so that the total number of 1 bits
    /// (including the parity bit) is even. If the count is already
    /// even, the parity bit is 0; if odd, the parity bit is 1.
    Even,

    /// Mark parity (parity bit always 1)
    ///
    /// The parity bit is always set to 1, regardless of the data.
    /// This is sometimes used for testing or specific protocols.
    Mark,

    /// Space parity (parity bit always 0)
    ///
    /// The parity bit is always set to 0, regardless of the data.
    /// This is sometimes used for testing or specific protocols.
    Space,
}

impl fmt::Display for Parity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Parity::None => write!(f, "None"),
            Parity::Odd => write!(f, "Odd"),
            Parity::Even => write!(f, "Even"),
            Parity::Mark => write!(f, "Mark"),
            Parity::Space => write!(f, "Space"),
        }
    }
}

/// Number of stop bits used to signal the end of a character.
///
/// Stop bits provide a pause between characters, allowing the receiver
/// to prepare for the next character. The number of stop bits affects
/// the total time per character transmission.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StopBits {
    /// One stop bit
    ///
    /// Most common setting. Provides adequate separation between
    /// characters for most applications.
    One,

    /// One and a half stop bits
    ///
    /// Rarely used. Provides a compromise between speed and reliability.
    /// Mainly used with 5-bit data transmission.
    OnePointFive,

    /// Two stop bits
    ///
    /// Provides extra time between characters. Used in situations
    /// where the receiver needs more time to process each character
    /// or when transmission errors are more likely.
    Two,
}

impl fmt::Display for StopBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            StopBits::One => write!(f, "1"),
            StopBits::OnePointFive => write!(f, "1.5"),
            StopBits::Two => write!(f, "2"),
        }
    }
}
