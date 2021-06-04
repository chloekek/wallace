//! File system functions that are not provided by the `std` crate.

#![doc(html_favicon_url = "../../../marketing/logo.svg")]
#![doc(html_logo_url = "../../../marketing/logo.svg")]

pub use self::fcntl::*;
pub use self::linkat::*;
pub use self::mknod::*;
pub use self::openat::*;

mod fcntl;
mod linkat;
mod mknod;
mod openat;
