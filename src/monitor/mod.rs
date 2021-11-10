/*
Copyright 2021 David Karrick

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
<LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
option. This file may not be copied, modified, or distributed
except according to those terms.
*/
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
/// Directory containing all backlight devices.
pub const BACKLIGHT_DIR: &str = "/sys/class/backlight";

#[cfg(target_os = "linux")]
pub use self::linux::{BackLightType, MonitorDevice};
