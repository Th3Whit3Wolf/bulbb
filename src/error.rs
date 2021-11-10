/*
Copyright 2021 David Karrick

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
<LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
option. This file may not be copied, modified, or distributed
except according to those terms.
*/
use std::{error, fmt, io, num};

use crate::monitor::BACKLIGHT_DIR;

#[cfg(feature = "dbus")]
use zbus::Error as ZBusError;

/// The error type for this crate.
///
/// The various errors that can be reported by this crate.
#[derive(Debug)]
#[non_exhaustive]
#[cfg(feature = "dbus")]
pub enum Error {
    /// Encounter error while setting brightness throught D-Bus.
    SetBrightnessDBusError(ZBusError),
    /// Brightness was set to invalid value.
    InvalidBrightnessLevel { given: u32, max: u32 },
    /// An I/O error.
    Io(io::Error),
    /// Error parsing `/sys/class/<backlight>` or `/sys/class/<leds>`
    /// brightness files.
    ParseBrightnessError(num::ParseIntError),
    /// Invalid device name.
    InvalidDeviceName { device: String },
}

/// The error type for this crate.
///
/// The various errors that can be reported by this crate.
#[derive(Debug)]
#[non_exhaustive]
#[cfg(not(feature = "dbus"))]
pub enum Error {
    /// Brightness was set to invalid value.
    InvalidBrightnessLevel { given: u32, max: u32 },
    /// An I/O error.
    Io(io::Error),
    /// Error parsing `/sys/class/<backlight>` or `/sys/class/<leds>`
    /// brightness files.
    ParseBrightnessError(num::ParseIntError),
    /// Invalid device name.
    InvalidDeviceName { device: String },
}

impl error::Error for Error {
    #[cfg(feature = "dbus")]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::SetBrightnessDBusError(e) => Some(e),
            Error::InvalidBrightnessLevel { given: _, max: _ } => None,
            Error::Io(e) => Some(e),
            Error::ParseBrightnessError(e) => Some(e),
            Error::InvalidDeviceName { device: _ } => None,
        }
    }
    #[cfg(not(feature = "dbus"))]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::InvalidBrightnessLevel { given: _, max: _ } => None,
            Error::Io(e) => Some(e),
            Error::ParseBrightnessError(e) => Some(e),
            Error::InvalidDeviceName { device: _ } => None,
        }
    }
}

impl fmt::Display for Error {
    #[cfg(feature = "dbus")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SetBrightnessDBusError(e) => write!(f, "address error: {}", e),
            Error::InvalidBrightnessLevel { given, max } => write!(
                f,
                "Invalid Brightness Level: expected number between 0 and {} but received {}.",
                max, given
            ),
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::ParseBrightnessError(e) => write!(f, "Unexpected Brightness {}", e),
            Error::InvalidDeviceName { device } => write!(
                f,
                "Invalid Device Name: {}/{}/ doest not exist.",
                BACKLIGHT_DIR, device
            ),
        }
    }
    #[cfg(not(feature = "dbus"))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidBrightnessLevel { given, max } => write!(
                f,
                "Invalid Brightness Level: expected number between 0 and {} but received {}.",
                max, given
            ),
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::ParseBrightnessError(e) => write!(f, "Unexpected Brightness {}", e),
            Error::InvalidDeviceName { device } => write!(
                f,
                "Invalid Device Name: {}/{}/ doest not exist.",
                BACKLIGHT_DIR, device
            ),
        }
    }
}

impl From<io::Error> for Error {
    fn from(val: io::Error) -> Self {
        Error::Io(val)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(val: num::ParseIntError) -> Self {
        Error::ParseBrightnessError(val)
    }
}

#[cfg(feature = "dbus")]
impl From<ZBusError> for Error {
    fn from(val: ZBusError) -> Self {
        Error::SetBrightnessDBusError(val)
    }
}
