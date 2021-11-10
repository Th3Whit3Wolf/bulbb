/*
Copyright 2021 David Karrick

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
<LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
option. This file may not be copied, modified, or distributed
except according to those terms.
*/

use std::{fmt, fs, path::Path};

#[cfg(not(feature = "dbus"))]
use std::{fs::OpenOptions, io::prelude::*};

use super::BACKLIGHT_DIR;
use crate::{
    error::Error,
    utils::{read_sys_backlight, SysBacklightInterface},
};

#[cfg(feature = "dbus")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "dbus")]
use zbus::Connection;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "dbus", derive(Serialize, Deserialize))]
/// Monitor Device information.
///
/// Devices are extracted from the `/sys/class/backlight/` directory.
pub struct MonitorDevice {
    /// Value taken from `/sys/class/backlight/<backlight>`.
    ///
    /// Name of the backlight device
    pub device: String,
    /// Value taken from
    /// [`/sys/class/backlight/<backlight>/bl_power`](https://www.kernel.org/doc/html/latest/admin-guide/abi-stable.html#abi-sys-class-backlight-backlight-bl-power).
    ///
    /// Controls the power of `<backlight>`.
    pub bl_power: u32,
    /// Value taken from
    /// [`/sys/class/backlight/<backlight>/brightness`](https://www.kernel.org/doc/html/latest/admin-guide/abi-stable.html#abi-sys-class-backlight-backlight-brightness).
    ///
    /// Control the brightness for this `<backlight>`. Values
    /// are between 0 and [max_brightness](struct.MonitorDevice.html#structfield.max_brightness).
    /// This file will also show the brightness level stored in the driver, which
    /// may not be the actual brightness (see [actual_brightness](struct.MonitorDevice.html#structfield.actual_brightness)).
    pub brightness: u32,
    /// Value taken from
    /// [`/sys/class/backlight/<backlight>/actual_brightness`](https://www.kernel.org/doc/html/latest/admin-guide/abi-stable.html#abi-sys-class-backlight-backlight-actual-brightness).
    ///
    /// Show the actual brightness by querying the hardware.
    pub actual_brightness: u32,
    /// Value taken from
    /// [`/sys/class/backlight/<backlight>/max_brightness`](https://www.kernel.org/doc/html/latest/admin-guide/abi-stable.html#abi-sys-class-backlight-backlight-max-brightness).
    ///
    /// Maximum brightness for `<backlight>`.
    pub max_brightness: u32,
    /// Value taken from
    /// [`/sys/class/backlight/<backlight>/type`](https://www.kernel.org/doc/html/latest/admin-guide/abi-stable.html#abi-sys-class-backlight-backlight-type).
    ///
    /// The type of interface controlled by `<backlight>`.
    pub bl_type: BackLightType,
}

/// The type of interface controlled by [`<backlight>`](struct.MonitorDevice.html).
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "dbus", derive(Serialize, Deserialize))]
pub enum BackLightType {
    /// The driver uses a standard firmware interface
    FirmWare,
    /// The driver uses a platform-specific interface
    PlatForm,
    /// The driver controls hardware registers directly
    Raw,
}

impl From<&BackLightType> for &str {
    fn from(val: &BackLightType) -> &'static str {
        match val {
            BackLightType::FirmWare => "Firmware",
            BackLightType::PlatForm => "Platform",
            BackLightType::Raw => "Raw",
        }
    }
}

impl From<BackLightType> for &str {
    fn from(val: BackLightType) -> &'static str {
        match val {
            BackLightType::FirmWare => "Firmware",
            BackLightType::PlatForm => "Platform",
            BackLightType::Raw => "Raw",
        }
    }
}

impl From<&BackLightType> for String {
    fn from(val: &BackLightType) -> String {
        match val {
            BackLightType::FirmWare => String::from("Firmware"),
            BackLightType::PlatForm => String::from("Platform"),
            BackLightType::Raw => String::from("Raw"),
        }
    }
}

impl From<BackLightType> for String {
    fn from(val: BackLightType) -> String {
        match val {
            BackLightType::FirmWare => String::from("Firmware"),
            BackLightType::PlatForm => String::from("Platform"),
            BackLightType::Raw => String::from("Raw"),
        }
    }
}

impl fmt::Display for BackLightType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            BackLightType::FirmWare => write!(f, "Firmware"),
            BackLightType::PlatForm => write!(f, "Platform"),
            BackLightType::Raw => write!(f, "Raw"),
        }
    }
}

impl MonitorDevice {
    /// Get monitor by device name.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::monitor::MonitorDevice;
    ///
    /// let device_name = format!("amdgpu_bl0");
    /// let monitor = MonitorDevice::get_monitor_device(device_name.clone()).unwrap();
    /// assert_eq!(monitor.get_device_name(), device_name);
    /// ```
    pub fn get_monitor_device(device: String) -> Result<MonitorDevice, Error> {
        if Path::new(format!("{}/{}", BACKLIGHT_DIR, &device).as_str()).is_dir() {
            let bl_power =
                read_sys_backlight(&device, SysBacklightInterface::Power)?.parse::<u32>()?;
            let brightness =
                read_sys_backlight(&device, SysBacklightInterface::Brightness)?.parse::<u32>()?;
            let actual_brightness =
                read_sys_backlight(&device, SysBacklightInterface::ActualBrightness)?
                    .parse::<u32>()?;
            let max_brightness = read_sys_backlight(&device, SysBacklightInterface::MaxBrightness)?
                .parse::<u32>()?;
            let bl_type = match read_sys_backlight(&device, SysBacklightInterface::Type)?.as_str() {
                "firmware" => BackLightType::FirmWare,
                "platform" => BackLightType::PlatForm,
                "raw" => BackLightType::Raw,
                _ => unreachable!(),
            };

            Ok(MonitorDevice {
                device,
                bl_power,
                brightness,
                actual_brightness,
                max_brightness,
                bl_type,
            })
        } else {
            Err(Error::InvalidDeviceName { device })
        }
    }

    /// Get all monitor devices.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::monitor::MonitorDevice;
    ///
    /// let monitors = MonitorDevice::get_all_monitor_devices().unwrap();
    /// for monitor in monitors {
    ///     println!("Monitor: {:?}", monitor);
    /// }
    /// ```
    pub fn get_all_monitor_devices() -> Result<Vec<MonitorDevice>, Error> {
        let mut monitors = Vec::with_capacity(1);

        if Path::new(BACKLIGHT_DIR).is_dir() {
            for device in fs::read_dir(BACKLIGHT_DIR)? {
                let device = device?;
                let device_name = device.file_name().into_string().unwrap();

                match MonitorDevice::get_monitor_device(device_name) {
                    Ok(dev) => monitors.push(dev),
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(monitors)
    }

    /// Get device name of monitor.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::monitor::MonitorDevice;
    ///
    /// let monitors = MonitorDevice::get_all_monitor_devices().unwrap();
    /// for monitor in monitors {
    ///     let max_brightness = monitor.get_max_brightness();
    ///     let device = monitor.get_device_name();
    ///     println!("Device: {}", device);
    ///     assert!(!device.is_empty())
    /// }
    /// ```
    pub fn get_device_name(&self) -> &str {
        &self.device
    }

    /// Get power of monitor.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::monitor::MonitorDevice;
    ///
    /// let monitors = MonitorDevice::get_all_monitor_devices().unwrap();
    /// for monitor in monitors {
    ///     let power = monitor.get_power();
    ///     println!("Power: {}", power);
    /// }
    /// ```
    pub fn get_power(&self) -> u32 {
        self.bl_power
    }

    /// Get brightness of monitor.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::monitor::MonitorDevice;
    ///
    /// let monitors = MonitorDevice::get_all_monitor_devices().unwrap();
    /// for monitor in monitors {
    ///     let max_brightness = monitor.get_max_brightness();
    ///     let brightness = monitor.get_brightness();
    ///     println!("Brightness: {}", brightness);
    ///     assert!(brightness >= 0 && brightness <= max_brightness)
    /// }
    /// ```
    pub fn get_brightness(&self) -> u32 {
        self.brightness
    }

    /// Get actual brightness of monitor
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::monitor::MonitorDevice;
    ///
    /// let monitors = MonitorDevice::get_all_monitor_devices().unwrap();
    /// for monitor in monitors {
    ///     let max_brightness = monitor.get_max_brightness();
    ///     let brightness = monitor.get_actual_brightness();
    ///     println!("Brightness: {}", brightness);
    ///     assert!(brightness >= 0 && brightness <= max_brightness)
    /// }
    /// ```
    pub fn get_actual_brightness(&self) -> u32 {
        self.actual_brightness
    }

    /// Get the maximum brightness value of monitor.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::monitor::MonitorDevice;
    ///
    /// let monitors = MonitorDevice::get_all_monitor_devices().unwrap();
    /// for monitor in monitors {
    ///     let max_brightness = monitor.get_max_brightness();
    ///     println!("Max Brightness: {}", max_brightness);
    ///     assert!(max_brightness > 0)
    /// }
    /// ```
    pub fn get_max_brightness(&self) -> u32 {
        self.max_brightness
    }

    /// Get the backlight type of monitor.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::monitor::MonitorDevice;
    ///
    /// let monitors = MonitorDevice::get_all_monitor_devices().unwrap();
    /// for monitor in monitors {
    ///     let mon_type = monitor.get_type();
    ///     println!("Type: {}", mon_type);
    /// }
    /// ```
    pub fn get_type(&self) -> BackLightType {
        self.bl_type
    }

    /// Set brightness of monitor.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::monitor::MonitorDevice;
    ///
    /// let monitors = MonitorDevice::get_all_monitor_devices().unwrap();
    /// monitors[0].set_brightness(20);
    /// ```
    #[cfg(feature = "dbus")]
    pub fn set_brightness(&self, level: u32) -> Result<(), Error> {
        if level <= self.max_brightness {
            let sd_bus = Connection::new_system().unwrap();
            match sd_bus.call_method(
                Some("org.freedesktop.login1"),
                "/org/freedesktop/login1/session/auto",
                Some("org.freedesktop.login1.Session"),
                "SetBrightness",
                &("backlight", &self.device, level),
            ) {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::SetBrightnessDBusError(e)),
            }
        } else {
            Err(Error::InvalidBrightnessLevel {
                given: level,
                max: self.max_brightness,
            })
        }
    }

    /// Set brightness of monitor.
    ///
    /// ### NOTE
    ///
    /// This method writes to `/sys/class/backlight/<backlight>/brightness`
    /// and will fail if user is not root (even when executed with sudo).
    /// It is recommended to create a udev rule to allow user of a certain
    /// group to write to the file. The example below will allow all users in
    /// the `video` group to change the brightness of all devices in `/sys/class/backlight/`.
    ///
    /// ```ignore
    /// ACTION=="add", SUBSYSTEM=="backlight", RUN+="/bin/chgrp video /sys/class/backlight/%k/brightness"
    /// ACTION=="add", SUBSYSTEM=="backlight", RUN+="/bin/chmod g+w /sys/class/backlight/%k/brightness"
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::monitor::MonitorDevice;
    ///
    /// let monitors = MonitorDevice::get_all_monitor_devices().unwrap();
    /// monitors[0].set_brightness(20);
    /// ```
    #[cfg(not(feature = "dbus"))]
    pub fn set_brightness(&self, level: u32) -> Result<(), Error> {
        if level <= self.max_brightness {
            // write to /sys/class/backlight/<backlight>/brightness
            let mut brightness = OpenOptions::new()
                .write(true)
                .open(format!("{}/{}/brightness", BACKLIGHT_DIR, &self.device))?;
            match brightness.write_all(level.to_string().as_bytes()) {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::Io(e)),
            }
        } else {
            Err(Error::InvalidBrightnessLevel {
                given: level,
                max: self.max_brightness,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_monitor_device(bl: MonitorDevice) {
        println!(
            "
Device: {}
	Type: {}
	Power: {}
	Brightness
\tMax: {}
\tActual: {}
\tCurrent: {}
		",
            bl.device,
            String::from(&bl.bl_type),
            bl.bl_power,
            bl.max_brightness,
            bl.actual_brightness,
            bl.brightness
        );
    }

    #[test]
    fn get_all_monitor_devices() {
        let monitors = MonitorDevice::get_all_monitor_devices().unwrap();
        for monitor in monitors {
            format_monitor_device(monitor)
        }
    }

    #[test]
    fn set_monitor() {
        let mut monitors = MonitorDevice::get_all_monitor_devices().unwrap();
        if !monitors.is_empty() {
            let first_device = monitors.remove(0);
            let device_name = first_device.get_device_name();
            let starting_brightness = first_device.get_actual_brightness();
            let max_brightness = first_device.get_max_brightness();
            let new_brightness: u32 = if starting_brightness > (max_brightness / 2) {
                0
            } else {
                max_brightness
            };

            let _set_brightness_1 =
                MonitorDevice::set_brightness(&first_device.clone(), new_brightness);

            let udpated_device =
                MonitorDevice::get_monitor_device(String::from(device_name)).unwrap();
            let updated_brightness = udpated_device.get_actual_brightness();

            assert_ne!(starting_brightness, updated_brightness);
            assert_eq!(updated_brightness, new_brightness);

            let _set_brightness_2 =
                MonitorDevice::set_brightness(&udpated_device, starting_brightness);

            let final_device =
                MonitorDevice::get_monitor_device(String::from(device_name)).unwrap();
            let final_brightness = final_device.get_actual_brightness();

            assert_ne!(final_brightness, updated_brightness);
            assert_eq!(final_brightness, starting_brightness);
        }
    }
}
