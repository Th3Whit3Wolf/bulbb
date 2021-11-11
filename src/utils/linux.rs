/*
Copyright 2021 David Karrick

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
<LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
option. This file may not be copied, modified, or distributed
except according to those terms.
*/
use std::{
    fs::{self},
    path::PathBuf,
};

use crate::{
    error::Error,
    misc::{LedDevice, LEDS_DIR},
    monitor::{MonitorDevice, BACKLIGHT_DIR},
};

pub enum SysBacklightInterface {
    Power,
    Brightness,
    ActualBrightness,
    MaxBrightness,
    Type,
}

pub fn read_sys_backlight(device: &str, info: SysBacklightInterface) -> Result<String, Error> {
    let mut path = PathBuf::new();
    path.push(BACKLIGHT_DIR);
    path.push(device);
    match info {
        SysBacklightInterface::Power => path.push("bl_power"),
        SysBacklightInterface::Brightness => path.push("brightness"),
        SysBacklightInterface::ActualBrightness => path.push("actual_brightness"),
        SysBacklightInterface::MaxBrightness => path.push("max_brightness"),
        SysBacklightInterface::Type => path.push("type"),
    }

    match fs::read_to_string(path) {
        Ok(s) => Ok(s.trim().to_string()),
        Err(e) => Err(Error::Io(e)),
    }
}

pub fn read_sys_led(device: &str, info: SysBacklightInterface) -> Result<String, Error> {
    let mut path = PathBuf::new();
    path.push(LEDS_DIR);
    path.push(device);
    match info {
        SysBacklightInterface::Power => path.push("bl_power"),
        SysBacklightInterface::Brightness => path.push("brightness"),
        SysBacklightInterface::ActualBrightness => path.push("actual_brightness"),
        SysBacklightInterface::MaxBrightness => path.push("max_brightness"),
        SysBacklightInterface::Type => path.push("type"),
    }

    match fs::read_to_string(path) {
        Ok(s) => Ok(s.trim().to_string()),
        Err(e) => Err(Error::Io(e)),
    }
}

pub fn format_monitor_device(bl: MonitorDevice) {
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

pub fn format_led_device(led: LedDevice) {
    let led_c = if let Some(c) = led.info.color {
        c.to_string()
    } else {
        String::from("")
    };
    let led_f = if let Some(f) = led.info.function {
        f.to_string()
    } else {
        String::from("")
    };

    println!(
        "
Device: {}
Info
\tDevice Name: {}
\tColor: {}
\tFunction: {}
Brightness
\tMax: {}
\tCurrent: {}
    ",
        led.info.device,
        led.info.device_name.unwrap_or_else(|| String::from("")),
        led_c,
        led_f,
        led.max_brightness,
        led.brightness
    );
}
