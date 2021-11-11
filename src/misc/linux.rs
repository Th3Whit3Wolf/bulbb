/*
Copyright 2021 David Karrick

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
<LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
option. This file may not be copied, modified, or distributed
except according to those terms.
*/
use std::{fmt, fs, path::Path};

use super::LEDS_DIR;
use crate::{
    error::Error,
    utils::{read_sys_led, SysBacklightInterface},
};

#[cfg(not(feature = "dbus"))]
use std::{fs::OpenOptions, io::prelude::*};

#[cfg(feature = "dbus")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "dbus")]
use zbus::Connection;

#[derive(Clone, Copy, Debug)]
pub struct LedFilterable<'a> {
    device_name: Option<&'a str>,
    color: Option<LedColor>,
    function: Option<LedFunction>,
}

impl<'a> LedFilterable<'a> {
    fn new() -> LedFilterable<'a> {
        LedFilterable {
            device_name: None,
            color: None,
            function: None,
        }
    }
    fn with_device_name(&'a mut self, device_name: &'a str) -> &'a mut LedFilterable {
        self.device_name = Some(device_name);
        self
    }
    fn with_color(&'a mut self, color: LedColor) -> &'a mut LedFilterable {
        self.color = Some(color);
        self
    }
    fn with_function(&'a mut self, function: LedFunction) -> &'a mut LedFilterable {
        self.function = Some(function);
        self
    }
    fn finish(&'a mut self) -> LedFilterable {
        *(self)
    }
    fn filter_by_device_name(&'a self, to_be_filtered: &str) -> bool {
        if let Some(device_name) = &self.device_name {
            to_be_filtered.contains(device_name)
        } else {
            false
        }
    }
    fn filter_by_color(&'a self, to_be_filtered: &str) -> bool {
        if let Some(color) = &self.color {
            to_be_filtered.contains(color.to_string().as_str())
        } else {
            false
        }
    }
    fn filter_by_function(&'a self, pre_filter: &str) -> bool {
        if let Some(function) = &self.function {
            pre_filter.contains(function.to_string().as_str())
        } else {
            false
        }
    }
    fn filter(&'a self, to_be_filtered: &str) -> bool {
        self.filter_by_device_name(to_be_filtered)
            || self.filter_by_color(to_be_filtered)
            || self.filter_by_function(to_be_filtered)
    }
}

fn multi_filter_led(filters: &[LedFilterable], to_be_filtered: &str) -> bool {
    let mut status = false;
    for f in filters {
        if f.filter(to_be_filtered) {
            status = true;
        }
    }
    status
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "dbus", derive(Serialize, Deserialize))]
/// LED device information
///
/// Devices are extracted from the `/sys/class/leds/` directory.
pub struct LedDevice {
    pub info: LedInfo,
    /** Set the brightness of the LED.

    Most LEDs don't have hardware brightness support, so will
    just be turned on for non-zero brightness settings.

    # Note

    > For multicolor LEDs, writing to this file will update all
    > LEDs within the group to a calculated percentage of what
    > each color LED intensity is set to.
    >
    > The percentage is calculated for each grouped LED via
    > the equation below::
    >
    > `led_brightness = brightness * multi_intensity/max_brightness`
    >
    > For additional details please refer to
    > [Multicolor LED handling under Linux](https://www.kernel.org/doc/html/latest/leds/leds-class-multicolor.html).

    The value is between 0 and [max_brightness](struct.LedDevice.html#structfield.max_brightness).

    Writing 0 to this file clears active trigger.

    Writing non-zero to this file while trigger is active changes the
    top brightness trigger is going to use. */
    pub brightness: u32,
    /** Maximum brightness level for this LED, default is 255 (LED_FULL).

    If the LED does not support different brightness levels, this
    should be 1. */
    pub max_brightness: u32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "dbus", derive(Serialize, Deserialize))]
/// LED Information.
pub struct LedInfo {
    /// **LED Device Naming**
    ///
    /// Is currently of the form:
    ///
    /// > “devicename:color:function”
    pub device: String,
    /**
    This should refer to a unique identifier created by the kernel,
    like e.g. phyN for network devices or inputN for input devices,
    rather than to the hardware. The information related to the product
    and the bus to which given device is hooked is available in sysfs.
    Generally this section is expected mostly for LEDs that are somehow associated with other devices.*/
    pub device_name: Option<String>,
    /// One of LED_COLOR_ID_* definitions from the header
    /// [include/dt-bindings/leds/common.h](https://github.com/torvalds/linux/blob/master/include/dt-bindings/leds/common.h).
    pub color: Option<LedColor>,
    /// One of LED_FUNCTION_* definitions from the header
    /// [include/dt-bindings/leds/common.h](https://github.com/torvalds/linux/blob/master/include/dt-bindings/leds/common.h).
    pub function: Option<LedFunction>,
}

impl LedDevice {
    pub fn get_led_devices_with_filter(f: LedFilterable) -> Result<Vec<LedDevice>, Error> {
        if Path::new(LEDS_DIR).is_dir() {
            fs::read_dir(LEDS_DIR)
                .unwrap()
                .into_iter()
                .filter(|r| r.is_ok()) // Get rid of Err variants for Result<DirEntry>
                .map(|r| r.unwrap().file_name().into_string()) // This is safe, since we only have the Ok variants
                .filter(|r| r.is_ok())
                .map(|r| r.unwrap()) // This is safe, since we only have the Ok variants
                // Get rid of Err variants for Result<DirEntry>
                .filter(|e| {
                    f.filter_by_device_name(e) || f.filter_by_color(e) || f.filter_by_function(e)
                })
                .map(LedDevice::get_led_device)
                .collect::<Result<Vec<LedDevice>, Error>>()
        } else {
            Ok(Vec::new())
        }
    }

    pub fn get_led_devices_with_multi_filter(f: &[LedFilterable]) -> Result<Vec<LedDevice>, Error> {
        if Path::new(LEDS_DIR).is_dir() {
            fs::read_dir(LEDS_DIR)
                .unwrap()
                .into_iter()
                .filter(|r| r.is_ok()) // Get rid of Err variants for Result<DirEntry>
                .map(|r| r.unwrap().file_name().into_string()) // This is safe, since we only have the Ok variants
                .filter(|r| r.is_ok())
                .map(|r| r.unwrap()) // This is safe, since we only have the Ok variants
                // Get rid of Err variants for Result<DirEntry>
                .filter(|e| multi_filter_led(f, e))
                .map(LedDevice::get_led_device)
                .collect::<Result<Vec<LedDevice>, Error>>()
        } else {
            Ok(Vec::new())
        }
    }
    /// Get LED by device name.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::misc::LedDevice;
    ///
    /// let device_name = format!("asus::kbd_backlight");
    /// let led_device = LedDevice::get_led_device(device_name.clone()).unwrap();
    /// assert_eq!(led_device.get_device_name(), device_name);
    /// ```
    pub fn get_led_device(device: String) -> Result<LedDevice, Error> {
        if Path::new(format!("{}/{}", LEDS_DIR, &device).as_str()).is_dir() {
            let brightness =
                read_sys_led(&device, SysBacklightInterface::Brightness)?.parse::<u32>()?;
            let max_brightness =
                read_sys_led(&device, SysBacklightInterface::MaxBrightness)?.parse::<u32>()?;
            let info = LedInfo::from_string(device);

            Ok(LedDevice {
                info,
                brightness,
                max_brightness,
            })
        } else {
            Err(Error::InvalidDeviceName { device })
        }
    }

    /// Get all LED devices.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::misc::LedDevice;
    ///
    /// let led_devices = LedDevice::get_all_led_devices().unwrap();
    /// for ld in led_devices {
    ///     println!("LED Device: {:?}", ld);
    /// }
    /// ```
    pub fn get_all_led_devices() -> Result<Vec<LedDevice>, Error> {
        let mut leds = Vec::with_capacity(1);

        if Path::new(LEDS_DIR).is_dir() {
            for device in fs::read_dir(LEDS_DIR)? {
                let device = device?;
                let device_name = device.file_name().into_string().unwrap();

                match LedDevice::get_led_device(device_name) {
                    Ok(dev) => leds.push(dev),
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(leds)
    }

    /// Get all keyboards devices.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::misc::LedDevice;
    ///
    /// let keyboards = LedDevice::get_all_keyboard_devices().unwrap();
    /// for keyboard in keyboards {
    ///     println!("Keyboard: {:?}", keyboard);
    /// }
    /// ```
    pub fn get_all_keyboard_devices() -> Result<Vec<LedDevice>, Error> {
        LedDevice::get_led_devices_with_filter(LedFilterable {
            device_name: None,
            color: None,
            function: Some(LedFunction::KbdBacklight),
        })
    }

    /// Get name of LED device.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::misc::LedDevice;
    ///
    /// let led_devices = LedDevice::get_all_led_devices().unwrap();
    /// for led_device in led_devices {
    ///     let max_brightness = led_device.get_max_brightness();
    ///     let device = led_device.get_device_name();
    ///     println!("Device: {}", device);
    ///     assert!(!device.is_empty())
    /// }
    /// ```
    pub fn get_device_name(&self) -> &str {
        &self.info.device
    }

    /// Get brightness of LED.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::misc::LedDevice;
    ///
    /// let led_devices = LedDevice::get_all_led_devices().unwrap();
    /// for led_device in led_devices {
    ///     let max_brightness = led_device.get_max_brightness();
    ///     let brightness = led_device.get_brightness();
    ///     println!("Brightness: {}", brightness);
    ///     assert!(brightness >= 0 && brightness <= max_brightness)
    /// }
    /// ```
    pub fn get_brightness(&self) -> u32 {
        self.brightness
    }

    /// Get the maximum brightness value of LED.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulbb::misc::LedDevice;
    ///
    /// let led_devices = LedDevice::get_all_led_devices().unwrap();
    /// for led_device in led_devices {
    ///     let max_brightness = led_device.get_max_brightness();
    ///     println!("Max Brightness: {}", max_brightness);
    ///     assert!(max_brightness > 0)
    /// }
    /// ```
    pub fn get_max_brightness(&self) -> u32 {
        self.max_brightness
    }

    /// Set brightness of LED.
    ///
    /// ### Examples
    ///
    /// ```
    /// use bulbb::misc::LedDevice;
    ///
    /// let keyboard = LedDevice::get_all_led_devices().unwrap();
    /// keyboard[0].set_brightness(20);
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
                &("leds", &self.info.device, level),
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

    /// Set brightness of led device.
    ///
    /// ### NOTE
    ///
    /// This method writes to `/sys/class/leds/<led>/brightness`
    /// and will fail if user is not root (even when executed with sudo).
    /// It is recommended to create a udev rule to allow user of a certain
    /// group to write to the file. The example below will allow all users in
    /// the `input` group to change the brightness of all devices in `/sys/class/leds/`.
    ///
    /// ```ignore
    /// ACTION=="add", SUBSYSTEM=="leds", RUN+="/bin/chgrp input /sys/class/leds/%k/brightness"
    /// ACTION=="add", SUBSYSTEM=="leds", RUN+="/bin/chmod g+w /sys/class/leds/%k/brightness"
    /// ```
    ///
    /// ### Examples
    ///
    /// ```
    /// use bulbb::misc::LedDevice;
    ///
    /// let keyboard = LedDevice::get_all_led_devices().unwrap();
    /// keyboard[0].set_brightness(20);
    /// ```
    #[cfg(not(feature = "dbus"))]
    pub fn set_brightness(&self, level: u32) -> Result<(), Error> {
        if level <= self.max_brightness {
            // write to /sys/class/leds/<led>/brightness
            let mut brightness = OpenOptions::new()
                .write(true)
                .open(format!("{}/{}/brightness", LEDS_DIR, &self.info.device))?;
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

impl LedInfo {
    /// Trys to parse string into LedInfo.
    pub fn from_string(s: String) -> LedInfo {
        let device = s.clone();
        let mut led_info = s.split(':').collect::<Vec<&str>>();
        led_info.retain(|&x| !x.is_empty());

        if led_info.len() == 3 {
            LedInfo {
                device,
                device_name: Some(led_info[0].to_string()),
                color: LedColor::from_id(led_info[1]),
                function: LedFunction::from_id(led_info[2]),
            }
        } else {
            let mut device_name: Option<String> = None;
            let mut color: Option<LedColor> = None;
            let mut function: Option<LedFunction> = None;

            let mut idx = 0_usize;
            while idx <= led_info.len() && !led_info.is_empty() {
                if LedColor::from_id(led_info[idx]).is_some() {
                    color = LedColor::from_id(led_info.remove(idx))
                } else if LedFunction::from_id(led_info[idx]).is_some() {
                    function = LedFunction::from_id(led_info.remove(idx))
                } else if !led_info.is_empty() {
                    device_name = Some(led_info.remove(idx).to_string())
                } else {
                    idx += 1
                }
            }

            LedInfo {
                device,
                device_name,
                color,
                function,
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "dbus", derive(Serialize, Deserialize))]
/// Color of LED.
pub enum LedColor {
    White,
    Red,
    Green,
    Blue,
    Amber,
    Violet,
    Yellow,
    Ir,
    Multi,
    Rgb,
    Max,
}

impl LedColor {
    /// Trys to parse str into LedColor.
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "white" => Some(LedColor::White),
            "red" => Some(LedColor::Red),
            "green" => Some(LedColor::Green),
            "blue" => Some(LedColor::Blue),
            "amber" => Some(LedColor::Amber),
            "violet" => Some(LedColor::Violet),
            "yellow" => Some(LedColor::Yellow),
            "ir" => Some(LedColor::Ir),
            "multi" => Some(LedColor::Multi),
            "rgb" => Some(LedColor::Rgb),
            "max" => Some(LedColor::Max),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "dbus", derive(Serialize, Deserialize))]
/// Function of the LED.
pub enum LedFunction {
    CapsLock,
    ScrollLock,
    NumLock,
    KbdBacklight,
    Power,
    Disk,
    Charging,
    Status,
    MicMute,
    Mute,
    Player1,
    Player2,
    Player3,
    Player4,
    Player5,
    Activity,
    Alarm,
    Backlight,
    Bluetooth,
    Boot,
    Cpu,
    Debug,
    DiskActivity,
    DiskErr,
    DiskRead,
    DiskWrite,
    Fault,
    Flash,
    Heartbeat,
    Indicator,
    Lan,
    Mail,
    Mtd,
    Panic,
    Programming,
    Rx,
    Sd,
    Standby,
    Torch,
    Tx,
    Usb,
    Wan,
    Wlan,
    Wps,
}

impl From<LedColor> for &str {
    fn from(val: LedColor) -> &'static str {
        match val {
            LedColor::White => "white",
            LedColor::Red => "red",
            LedColor::Green => "green",
            LedColor::Blue => "blue",
            LedColor::Amber => "amber",
            LedColor::Violet => "violet",
            LedColor::Yellow => "yellow",
            LedColor::Ir => "ir",
            LedColor::Multi => "multi",
            LedColor::Rgb => "rgb",
            LedColor::Max => "max",
        }
    }
}

impl From<&LedColor> for &str {
    fn from(val: &LedColor) -> &'static str {
        match val {
            LedColor::White => "white",
            LedColor::Red => "red",
            LedColor::Green => "green",
            LedColor::Blue => "blue",
            LedColor::Amber => "amber",
            LedColor::Violet => "violet",
            LedColor::Yellow => "yellow",
            LedColor::Ir => "ir",
            LedColor::Multi => "multi",
            LedColor::Rgb => "rgb",
            LedColor::Max => "max",
        }
    }
}

impl From<LedColor> for String {
    fn from(val: LedColor) -> String {
        match val {
            LedColor::White => String::from("white"),
            LedColor::Red => String::from("red"),
            LedColor::Green => String::from("green"),
            LedColor::Blue => String::from("blue"),
            LedColor::Amber => String::from("amber"),
            LedColor::Violet => String::from("violet"),
            LedColor::Yellow => String::from("yellow"),
            LedColor::Ir => String::from("ir"),
            LedColor::Multi => String::from("multi"),
            LedColor::Rgb => String::from("rgb"),
            LedColor::Max => String::from("max"),
        }
    }
}

impl From<&LedColor> for String {
    fn from(val: &LedColor) -> String {
        match val {
            LedColor::White => String::from("white"),
            LedColor::Red => String::from("red"),
            LedColor::Green => String::from("green"),
            LedColor::Blue => String::from("blue"),
            LedColor::Amber => String::from("amber"),
            LedColor::Violet => String::from("violet"),
            LedColor::Yellow => String::from("yellow"),
            LedColor::Ir => String::from("ir"),
            LedColor::Multi => String::from("multi"),
            LedColor::Rgb => String::from("rgb"),
            LedColor::Max => String::from("max"),
        }
    }
}

impl fmt::Display for LedColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            LedColor::White => write!(f, "white"),
            LedColor::Red => write!(f, "red"),
            LedColor::Green => write!(f, "green"),
            LedColor::Blue => write!(f, "blue"),
            LedColor::Amber => write!(f, "amber"),
            LedColor::Violet => write!(f, "violet"),
            LedColor::Yellow => write!(f, "yellow"),
            LedColor::Ir => write!(f, "ir"),
            LedColor::Multi => write!(f, "multi"),
            LedColor::Rgb => write!(f, "rgb"),
            LedColor::Max => write!(f, "max"),
        }
    }
}

impl LedFunction {
    /// Trys to parse str into LedFunction.
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "capslock" => Some(LedFunction::CapsLock),
            "scrolllock" => Some(LedFunction::ScrollLock),
            "numlock" => Some(LedFunction::NumLock),
            "kbd_backlight" => Some(LedFunction::KbdBacklight),
            "power" => Some(LedFunction::Power),
            "disk" => Some(LedFunction::Disk),
            "charging" => Some(LedFunction::Charging),
            "status" => Some(LedFunction::Status),
            "micmute" => Some(LedFunction::MicMute),
            "mute" => Some(LedFunction::Mute),
            "player-1" => Some(LedFunction::Player1),
            "player-2" => Some(LedFunction::Player2),
            "player-3" => Some(LedFunction::Player3),
            "player-4" => Some(LedFunction::Player4),
            "player-5" => Some(LedFunction::Player5),
            "activity" => Some(LedFunction::Activity),
            "alarm" => Some(LedFunction::Alarm),
            "backlight" => Some(LedFunction::Backlight),
            "bluetooth" => Some(LedFunction::Bluetooth),
            "boot" => Some(LedFunction::Boot),
            "cpu" => Some(LedFunction::Cpu),
            "debug" => Some(LedFunction::Debug),
            "disk-activity" => Some(LedFunction::DiskActivity),
            "disk-err" => Some(LedFunction::DiskErr),
            "disk-read" => Some(LedFunction::DiskRead),
            "disk-write" => Some(LedFunction::DiskWrite),
            "fault" => Some(LedFunction::Fault),
            "flash" => Some(LedFunction::Flash),
            "heartbeat" => Some(LedFunction::Heartbeat),
            "indicator" => Some(LedFunction::Indicator),
            "lan" => Some(LedFunction::Lan),
            "mail" => Some(LedFunction::Mail),
            "mtd" => Some(LedFunction::Mtd),
            "panic" => Some(LedFunction::Panic),
            "programming" => Some(LedFunction::Programming),
            "rx" => Some(LedFunction::Rx),
            "sd" => Some(LedFunction::Sd),
            "standby" => Some(LedFunction::Standby),
            "torch" => Some(LedFunction::Torch),
            "tx" => Some(LedFunction::Tx),
            "usb" => Some(LedFunction::Usb),
            "wan" => Some(LedFunction::Wan),
            "wlan" => Some(LedFunction::Wlan),
            "wps" => Some(LedFunction::Wps),
            _ => None,
        }
    }
}

impl From<&LedFunction> for &str {
    fn from(val: &LedFunction) -> &'static str {
        match val {
            LedFunction::CapsLock => "capslock",
            LedFunction::ScrollLock => "scrolllock",
            LedFunction::NumLock => "numlock",
            LedFunction::KbdBacklight => "kbd_backlight",
            LedFunction::Power => "power",
            LedFunction::Disk => "disk",
            LedFunction::Charging => "charging",
            LedFunction::Status => "status",
            LedFunction::MicMute => "micmute",
            LedFunction::Mute => "mute",
            LedFunction::Player1 => "player-1",
            LedFunction::Player2 => "player-2",
            LedFunction::Player3 => "player-3",
            LedFunction::Player4 => "player-4",
            LedFunction::Player5 => "player-5",
            LedFunction::Activity => "activity",
            LedFunction::Alarm => "alarm",
            LedFunction::Backlight => "backlight",
            LedFunction::Bluetooth => "bluetooth",
            LedFunction::Boot => "boot",
            LedFunction::Cpu => "cpu",
            LedFunction::Debug => "debug",
            LedFunction::DiskActivity => "disk-activity",
            LedFunction::DiskErr => "disk-err",
            LedFunction::DiskRead => "disk-read",
            LedFunction::DiskWrite => "disk-write",
            LedFunction::Fault => "fault",
            LedFunction::Flash => "flash",
            LedFunction::Heartbeat => "heartbeat",
            LedFunction::Indicator => "indicator",
            LedFunction::Lan => "lan",
            LedFunction::Mail => "mail",
            LedFunction::Mtd => "mtd",
            LedFunction::Panic => "panic",
            LedFunction::Programming => "programming",
            LedFunction::Rx => "rx",
            LedFunction::Sd => "sd",
            LedFunction::Standby => "standby",
            LedFunction::Torch => "torch",
            LedFunction::Tx => "tx",
            LedFunction::Usb => "usb",
            LedFunction::Wan => "wan",
            LedFunction::Wlan => "wlan",
            LedFunction::Wps => "wps",
        }
    }
}

impl From<LedFunction> for &str {
    fn from(val: LedFunction) -> &'static str {
        match val {
            LedFunction::CapsLock => "capslock",
            LedFunction::ScrollLock => "scrolllock",
            LedFunction::NumLock => "numlock",
            LedFunction::KbdBacklight => "kbd_backlight",
            LedFunction::Power => "power",
            LedFunction::Disk => "disk",
            LedFunction::Charging => "charging",
            LedFunction::Status => "status",
            LedFunction::MicMute => "micmute",
            LedFunction::Mute => "mute",
            LedFunction::Player1 => "player-1",
            LedFunction::Player2 => "player-2",
            LedFunction::Player3 => "player-3",
            LedFunction::Player4 => "player-4",
            LedFunction::Player5 => "player-5",
            LedFunction::Activity => "activity",
            LedFunction::Alarm => "alarm",
            LedFunction::Backlight => "backlight",
            LedFunction::Bluetooth => "bluetooth",
            LedFunction::Boot => "boot",
            LedFunction::Cpu => "cpu",
            LedFunction::Debug => "debug",
            LedFunction::DiskActivity => "disk-activity",
            LedFunction::DiskErr => "disk-err",
            LedFunction::DiskRead => "disk-read",
            LedFunction::DiskWrite => "disk-write",
            LedFunction::Fault => "fault",
            LedFunction::Flash => "flash",
            LedFunction::Heartbeat => "heartbeat",
            LedFunction::Indicator => "indicator",
            LedFunction::Lan => "lan",
            LedFunction::Mail => "mail",
            LedFunction::Mtd => "mtd",
            LedFunction::Panic => "panic",
            LedFunction::Programming => "programming",
            LedFunction::Rx => "rx",
            LedFunction::Sd => "sd",
            LedFunction::Standby => "standby",
            LedFunction::Torch => "torch",
            LedFunction::Tx => "tx",
            LedFunction::Usb => "usb",
            LedFunction::Wan => "wan",
            LedFunction::Wlan => "wlan",
            LedFunction::Wps => "wps",
        }
    }
}

impl From<&LedFunction> for String {
    fn from(val: &LedFunction) -> String {
        match val {
            LedFunction::CapsLock => String::from("capslock"),
            LedFunction::ScrollLock => String::from("scrolllock"),
            LedFunction::NumLock => String::from("numlock"),
            LedFunction::KbdBacklight => String::from("kbd_backlight"),
            LedFunction::Power => String::from("power"),
            LedFunction::Disk => String::from("disk"),
            LedFunction::Charging => String::from("charging"),
            LedFunction::Status => String::from("status"),
            LedFunction::MicMute => String::from("micmute"),
            LedFunction::Mute => String::from("mute"),
            LedFunction::Player1 => String::from("player-1"),
            LedFunction::Player2 => String::from("player-2"),
            LedFunction::Player3 => String::from("player-3"),
            LedFunction::Player4 => String::from("player-4"),
            LedFunction::Player5 => String::from("player-5"),
            LedFunction::Activity => String::from("activity"),
            LedFunction::Alarm => String::from("alarm"),
            LedFunction::Backlight => String::from("backlight"),
            LedFunction::Bluetooth => String::from("bluetooth"),
            LedFunction::Boot => String::from("boot"),
            LedFunction::Cpu => String::from("cpu"),
            LedFunction::Debug => String::from("debug"),
            LedFunction::DiskActivity => String::from("disk-activity"),
            LedFunction::DiskErr => String::from("disk-err"),
            LedFunction::DiskRead => String::from("disk-read"),
            LedFunction::DiskWrite => String::from("disk-write"),
            LedFunction::Fault => String::from("fault"),
            LedFunction::Flash => String::from("flash"),
            LedFunction::Heartbeat => String::from("heartbeat"),
            LedFunction::Indicator => String::from("indicator"),
            LedFunction::Lan => String::from("lan"),
            LedFunction::Mail => String::from("mail"),
            LedFunction::Mtd => String::from("mtd"),
            LedFunction::Panic => String::from("panic"),
            LedFunction::Programming => String::from("programming"),
            LedFunction::Rx => String::from("rx"),
            LedFunction::Sd => String::from("sd"),
            LedFunction::Standby => String::from("standby"),
            LedFunction::Torch => String::from("torch"),
            LedFunction::Tx => String::from("tx"),
            LedFunction::Usb => String::from("usb"),
            LedFunction::Wan => String::from("wan"),
            LedFunction::Wlan => String::from("wlan"),
            LedFunction::Wps => String::from("wps"),
        }
    }
}

impl From<LedFunction> for String {
    fn from(val: LedFunction) -> String {
        match val {
            LedFunction::CapsLock => String::from("capslock"),
            LedFunction::ScrollLock => String::from("scrolllock"),
            LedFunction::NumLock => String::from("numlock"),
            LedFunction::KbdBacklight => String::from("kbd_backlight"),
            LedFunction::Power => String::from("power"),
            LedFunction::Disk => String::from("disk"),
            LedFunction::Charging => String::from("charging"),
            LedFunction::Status => String::from("status"),
            LedFunction::MicMute => String::from("micmute"),
            LedFunction::Mute => String::from("mute"),
            LedFunction::Player1 => String::from("player-1"),
            LedFunction::Player2 => String::from("player-2"),
            LedFunction::Player3 => String::from("player-3"),
            LedFunction::Player4 => String::from("player-4"),
            LedFunction::Player5 => String::from("player-5"),
            LedFunction::Activity => String::from("activity"),
            LedFunction::Alarm => String::from("alarm"),
            LedFunction::Backlight => String::from("backlight"),
            LedFunction::Bluetooth => String::from("bluetooth"),
            LedFunction::Boot => String::from("boot"),
            LedFunction::Cpu => String::from("cpu"),
            LedFunction::Debug => String::from("debug"),
            LedFunction::DiskActivity => String::from("disk-activity"),
            LedFunction::DiskErr => String::from("disk-err"),
            LedFunction::DiskRead => String::from("disk-read"),
            LedFunction::DiskWrite => String::from("disk-write"),
            LedFunction::Fault => String::from("fault"),
            LedFunction::Flash => String::from("flash"),
            LedFunction::Heartbeat => String::from("heartbeat"),
            LedFunction::Indicator => String::from("indicator"),
            LedFunction::Lan => String::from("lan"),
            LedFunction::Mail => String::from("mail"),
            LedFunction::Mtd => String::from("mtd"),
            LedFunction::Panic => String::from("panic"),
            LedFunction::Programming => String::from("programming"),
            LedFunction::Rx => String::from("rx"),
            LedFunction::Sd => String::from("sd"),
            LedFunction::Standby => String::from("standby"),
            LedFunction::Torch => String::from("torch"),
            LedFunction::Tx => String::from("tx"),
            LedFunction::Usb => String::from("usb"),
            LedFunction::Wan => String::from("wan"),
            LedFunction::Wlan => String::from("wlan"),
            LedFunction::Wps => String::from("wps"),
        }
    }
}

impl fmt::Display for LedFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            LedFunction::CapsLock => write!(f, "capslock"),
            LedFunction::ScrollLock => write!(f, "scrolllock"),
            LedFunction::NumLock => write!(f, "numlock"),
            LedFunction::KbdBacklight => write!(f, "kbd_backlight"),
            LedFunction::Power => write!(f, "power"),
            LedFunction::Disk => write!(f, "disk"),
            LedFunction::Charging => write!(f, "charging"),
            LedFunction::Status => write!(f, "status"),
            LedFunction::MicMute => write!(f, "micmute"),
            LedFunction::Mute => write!(f, "mute"),
            LedFunction::Player1 => write!(f, "player-1"),
            LedFunction::Player2 => write!(f, "player-2"),
            LedFunction::Player3 => write!(f, "player-3"),
            LedFunction::Player4 => write!(f, "player-4"),
            LedFunction::Player5 => write!(f, "player-5"),
            LedFunction::Activity => write!(f, "activity"),
            LedFunction::Alarm => write!(f, "alarm"),
            LedFunction::Backlight => write!(f, "backlight"),
            LedFunction::Bluetooth => write!(f, "bluetooth"),
            LedFunction::Boot => write!(f, "boot"),
            LedFunction::Cpu => write!(f, "cpu"),
            LedFunction::Debug => write!(f, "debug"),
            LedFunction::DiskActivity => write!(f, "disk-activity"),
            LedFunction::DiskErr => write!(f, "disk-err"),
            LedFunction::DiskRead => write!(f, "disk-read"),
            LedFunction::DiskWrite => write!(f, "disk-write"),
            LedFunction::Fault => write!(f, "fault"),
            LedFunction::Flash => write!(f, "flash"),
            LedFunction::Heartbeat => write!(f, "heartbeat"),
            LedFunction::Indicator => write!(f, "indicator"),
            LedFunction::Lan => write!(f, "lan"),
            LedFunction::Mail => write!(f, "mail"),
            LedFunction::Mtd => write!(f, "mtd"),
            LedFunction::Panic => write!(f, "panic"),
            LedFunction::Programming => write!(f, "programming"),
            LedFunction::Rx => write!(f, "rx"),
            LedFunction::Sd => write!(f, "sd"),
            LedFunction::Standby => write!(f, "standby"),
            LedFunction::Torch => write!(f, "torch"),
            LedFunction::Tx => write!(f, "tx"),
            LedFunction::Usb => write!(f, "usb"),
            LedFunction::Wan => write!(f, "wan"),
            LedFunction::Wlan => write!(f, "wlan"),
            LedFunction::Wps => write!(f, "wps"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::format_led_device;

    #[test]
    fn parse_led_device_names() {
        let devices = vec![
            "asus::kbd_backlight",
            "input13::capslock",
            "input13::compose",
            "input13::kana",
            "input13::numlock",
            "input13::scrolllock",
            "input2::capslock",
            "input2::numlock",
            "input2::scrolllock",
            "phy0-led",
        ];
        for dev in devices {
            let led_info = LedInfo::from_string(dev.to_string());
            let led_c = if let Some(c) = led_info.color {
                c.to_string()
            } else {
                String::from("")
            };
            let led_f = if let Some(f) = led_info.function {
                f.to_string()
            } else {
                String::from("")
            };

            println!(
                "Device: {}\nDevice Name: {}\nColor: {}\nFunction: {}\n",
                led_info.device,
                led_info.device_name.unwrap_or_else(|| String::from("")),
                led_c,
                led_f
            )
        }
    }

    #[test]
    fn get_all_led_devices() {
        let leds = LedDevice::get_all_led_devices().unwrap();
        for led in leds {
            format_led_device(led)
        }
    }

    #[test]
    fn get_all_keyboard_devices() {
        let keyboards = LedDevice::get_all_keyboard_devices().unwrap();
        for kbd in keyboards {
            format_led_device(kbd)
        }
    }

    #[test]
    fn filter() {
        let filter1 = LedFilterable {
            device_name: Some("dev"),
            color: None,
            function: None,
        };

        let mut filter2 = LedFilterable::new();
        let filter2 = filter2.with_device_name("dev").finish();
        assert_eq!(filter1.device_name, filter2.device_name);
        assert_eq!(filter1.color.is_none(), filter2.color.is_none());
        assert_eq!(filter1.function.is_none(), filter2.function.is_none());
    }
}
