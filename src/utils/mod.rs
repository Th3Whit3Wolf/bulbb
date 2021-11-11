/*
Copyright 2021 David Karrick

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
<LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
option. This file may not be copied, modified, or distributed
except according to those terms.
*/
#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use self::linux::{
    format_led_device, format_monitor_device, read_sys_backlight, read_sys_led,
    SysBacklightInterface,
};
