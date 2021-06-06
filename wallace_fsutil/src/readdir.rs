use crate::Dir;
use std::ffi::CStr;
use std::io::Error;
use std::io::Result;
use std::marker::PhantomPinned;
use std::os::raw::c_char;
use std::os::raw::c_uchar;
use std::os::raw::c_ushort;
use std::pin::Pin;

/// Safe wrapper around `dirent`.
///
/// Because POSIX specifies that `d_name` should not be used as an lvalue,
/// we assume that the type is actually an unsized type.
/// Therefore this type do not impl [`Unpin`].
#[repr(transparent)]
pub struct Dirent
{
    inner: libc::dirent,
    _pinned: PhantomPinned,
}

impl Dirent
{
    pub fn d_ino   (&self) -> libc::ino_t { self.inner.d_ino    }
    pub fn d_off   (&self) -> libc::off_t { self.inner.d_off    }
    pub fn d_reclen(&self) -> c_ushort    { self.inner.d_reclen }
    pub fn d_type  (&self) -> c_uchar     { self.inner.d_type   }

    pub fn d_name(&self) -> &CStr
    {
        // SAFETY: d_name is guaranteed null-terminated.
        unsafe {
            CStr::from_ptr(&self.inner.d_name as *const c_char)
        }
    }
}

/// Perform the `readdir` system call.
pub fn readdir(dirp: &mut Dir) -> Result<Option<Pin<&Dirent>>>
{
    // SAFETY: Reading errno is safe.
    unsafe {
        // readdir returns NULL if the final entry has been reached,
        // but it also returns NULL if an error occurs.
        // To distinguish between these cases, set errno to zero first.
        *libc::__errno_location() = 0;

        // SAFETY: Dir ensures the DIR is alive.
        let dirent = libc::readdir(dirp.inner);

        if dirent.is_null() {

            if *libc::__errno_location() == 0 {
                Ok(None)
            } else {
                Err(Error::last_os_error())
            }

        } else {

            // SAFETY: Dirent is #[repr(transparent)].
            let dirent_ref = &*(dirent as *const Dirent);

            // SAFETY: Dirents can only be obtained through readdir,
            // and this does not allow you to move them.
            let dirent_pin = Pin::new_unchecked(dirent_ref);

            Ok(Some(dirent_pin))

        }
    }
}
