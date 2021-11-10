/*
Copyright 2021 David Karrick

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
<LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
option. This file may not be copied, modified, or distributed
except according to those terms.
*/

#![deny(rustdoc::broken_intra_doc_links)]

//! # Bulbb
//!
//! Boldly Universal Library for managing Backlight Brightness
//! This library allows the user to change the brightness of backlit devices in Linux.
//!
//! ## Goals
//!
//! * [ ] Mac Support
//! * [ ] Windows Support
//! * [ ] FreeBSD Support
//!
//! ## License
//!
//! This software is distributed under the terms of both the MIT license and the
//! Apache License (Version 2.0).
//!
//! See [LICENSE-APACHE](https://github.com/Th3Whit3Wolf/bulbb/blob/main/LICENSE-APACHE)(or <https://www.apache.org/licenses/LICENSE-2.0>) and
//! [LICENSE-MIT](https://github.com/Th3Whit3Wolf/bulbb/blob/main/LICENSE-MIT)(or <https://opensource.org/licenses/MIT>) for license details.
//!
//! ### Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally
//! submitted for inclusion in the work by you, as defined in the
//! Apache-2.0 license, shall be dual licensed as above, without any
//! additional terms or conditions.
//!
//! ## References
//!
//! Below is some of the reference material I used (stole from)
//! while writing the documentation for this crate.
//!
//! ### Linux
//!
//! * [Backlight (Linux ABI description)](https://www.kernel.org/doc/html/latest/admin-guide/abi-testing.html#file-srv-docbuild-lib-git-linux-testing-sysfs-class-led)
//! * [LEDs (Linux ABI description)](https://www.kernel.org/doc/html/latest/admin-guide/abi-stable.html#abi-file-stable-sysfs-class-backlight)
//!
//!

mod utils;

/// Possible errors for this crate.
pub mod error;
/// Get lighting of led(s)
pub mod misc;
/// Get backlighting of monitor(s)
pub mod monitor;
