//! It is best to think of a volume as an on-disk collection of _objects_,
//! where an object is a byte array identified by its hash.
//!
//! Objects can be insert into the volume using
//! the [`Volume::insert_from_file`] method.
//! They can later be retrieved using the [`Volume::get`] method.
//!
//! Internally, a volume is a directory on the file system.
//! Each object is stored as a separate file,
//! at a path derived from the hash of the object.

#![doc(html_favicon_url = "../../../marketing/logo.svg")]
#![doc(html_logo_url = "../../../marketing/logo.svg")]

pub use self::hash::*;
pub use self::volume::*;

mod hash;
mod volume;
