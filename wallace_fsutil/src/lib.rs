//! File system functions that are not provided by the `std` crate.

#![doc(html_favicon_url = "../../../marketing/logo.svg")]
#![doc(html_logo_url = "../../../marketing/logo.svg")]

pub use self::openat::*;
pub use self::renameat::*;

mod openat;
mod renameat;
