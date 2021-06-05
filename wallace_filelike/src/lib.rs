//! Read-only file-like access layer for objects.
//!
//! This crate implements a file system that exposes objects<sup>†</sup>.
//! Objects are automatically given paths based on metadata attached to them.
//! They can also be accessed directly by their object identifiers.
//! The file system is only exposed as a Rust API.
//! This crate does not implement integration with any
//! concrete operating system facility or file access protocol.
//! Such functionality may provided by other crates,
//! which in turn depend on this crate for the core functionality.
//! Possible integrations include FUSE, HTTP, and FTP.
//!
//! <sup>†</sup> For more information about objects,
//! see the [`wallace_volume`] crate.

#![doc(html_favicon_url = "../../../marketing/logo.svg")]
#![doc(html_logo_url = "../../../marketing/logo.svg")]

pub use parsed_path::*;

mod parsed_path;
