//! Functions for working with volumes.
//!
//! A volume is a collection of objects.
//! It is best to think of an object as an array of bytes.
//! Like an array of bytes, an object has a length N, and N bytes.
//! These bytes can be anything; the volume does not judge objects.
//! Associating objects with metadata is not the job of the volume.
//! Objects cannot be modified after they are inserted into a volume.
//!
//! # Content addressable storage
//!
//! Each object is identified by its hash.
//! The hash of an object is computed by feeding
//! the bytes that make up the object
//! through the SHA-256 hash function.
//! This happens automatically when inserting a new object
//! using the methods on the [`Volume`][`crate::Volume`] type.
//! Two objects made up of the same byte array
//! always have the exact same hash.
//!
//! # On-disk storage of objects
//!
//! A volume is stored a directory on the file system.
//! Each object exists as a regular file,
//! with a path in the volume’s directory,
//! named after the hash of the object.
//! For instance, an object whose hash is `315f5b...`
//! would be stored in a file at the path `315f5b...`
//! in the directory of the volume.
//! The contents of the file backing an object
//! are simply the bytes that make up that object.
//!
//! If a file backing an object has any additional hard links,
//! then those must not be used to alter the object!
//! Remember, objects cannot be modified
//! after they are inserted into a volume.
//! It is therefore recommended to delete any hard links.
//!
//! For more information on how files are inserted into a volume,
//! see the documentation on the insert methods on the [`Volume`] type.
//!
//! # How to use this crate
//!
//! Volumes can be manipulated through the methods on the [`Volume`] type.
//! See the documentation of these methods for more information.

#![doc(html_favicon_url = "../../../marketing/logo.svg")]
#![doc(html_logo_url = "../../../marketing/logo.svg")]

pub use self::hash::*;
pub use self::volume::*;

mod hash;
mod volume;

#[cfg(test)] use self::testdata::*;
#[cfg(test)] mod testdata;
