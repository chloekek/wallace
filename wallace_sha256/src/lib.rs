//! Implementation of SHA-256 based on libsodium.

#![doc(html_favicon_url = "../../../marketing/logo.svg")]
#![doc(html_logo_url = "../../../marketing/logo.svg")]

use std::io::Result;
use std::io::Write;
use std::mem::MaybeUninit;
use std::os::raw::c_int;
use std::os::raw::c_uchar;
use std::os::raw::c_ulonglong;

#[repr(C)]
#[derive(Clone)]
struct crypto_hash_sha256_state
{
    state: [u32; 8],
    count: u64,
    buf:   [u8; 64],
}

#[link(name = "sodium")]
extern "C"
{
    fn crypto_hash_sha256_init(
        state: *mut crypto_hash_sha256_state,
    ) -> c_int;

    fn crypto_hash_sha256_update(
        state: *mut crypto_hash_sha256_state,
        r#in:  *const c_uchar,
        inlen: c_ulonglong,
    ) -> c_int;

    fn crypto_hash_sha256_final(
        state: *mut crypto_hash_sha256_state,
        out:   *mut c_uchar,
    ) -> c_int;
}

/// SHA-256 digest with a multi-part interface.
///
/// The [`Write`] impl calls [`Sha256::update`] on writes.
/// This is especially convenient when using [`copy`][`std::io::copy`].
/// It never returns an error.
#[derive(Clone)]
pub struct Sha256
{
    inner: crypto_hash_sha256_state,
}

impl Sha256
{
    /// Create a new, empty digest.
    pub fn new() -> Self
    {
        unsafe {
            let mut inner = MaybeUninit::uninit();
            crypto_hash_sha256_init(inner.as_mut_ptr());
            Self{inner: inner.assume_init()}
        }
    }

    /// Update the digest using a buffer.
    pub fn update(&mut self, buf: &[u8])
    {
        unsafe {
            crypto_hash_sha256_update(
                &mut self.inner,
                buf.as_ptr(),
                buf.len() as u64,
            );
        }
    }

    /// Finalize the digest, returning the hash.
    pub fn finalize(mut self) -> [u8; 32]
    {
        unsafe {
            let mut buf = MaybeUninit::uninit();
            crypto_hash_sha256_final(
                &mut self.inner,
                buf.as_mut_ptr() as *mut u8,
            );
            buf.assume_init()
        }
    }
}

impl Write for Sha256
{
    fn write(&mut self, buf: &[u8]) -> Result<usize>
    {
        self.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()>
    {
        Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_example_hashes()
    {
        let table: &[(&[_], _)] = &[
            (b"",
             [0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14,
              0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
              0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c,
              0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55]),
            (b"Hello, world!",
             [0x31, 0x5f, 0x5b, 0xdb, 0x76, 0xd0, 0x78, 0xc4,
              0x3b, 0x8a, 0xc0, 0x06, 0x4e, 0x4a, 0x01, 0x64,
              0x61, 0x2b, 0x1f, 0xce, 0x77, 0xc8, 0x69, 0x34,
              0x5b, 0xfc, 0x94, 0xc7, 0x58, 0x94, 0xed, 0xd3]),
        ];
        for &(input, expected) in table {
            let mut sha256 = Sha256::new();
            sha256.update(input);
            let actual = sha256.finalize();
            assert_eq!(actual, expected);
        }
    }
}
