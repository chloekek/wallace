//! File system functions that are not provided by the `std` crate.

#![doc(html_favicon_url = "../../../marketing/logo.svg")]
#![doc(html_logo_url = "../../../marketing/logo.svg")]

pub use self::fcntl::*;
pub use self::fdopendir::*;
pub use self::linkat::*;
pub use self::mknod::*;
pub use self::openat::*;
pub use self::readdir::*;

mod fcntl;
mod fdopendir;
mod linkat;
mod mknod;
mod openat;
mod readdir;
