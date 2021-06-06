use std::io::Error;
use std::io::Result;
use std::os::unix::io::IntoRawFd;

/// Owned wrapper around `DIR`.
pub struct Dir
{
    pub (crate) inner: *mut libc::DIR,
}

impl Drop for Dir
{
    fn drop(&mut self)
    {
        unsafe {
            libc::closedir(self.inner);
        }
    }
}

/// Perform the `fdopendir` system call.
pub fn fdopendir(fd: impl IntoRawFd) -> Result<Dir>
{
    // SAFETY: This function just takes an integer.
    let dir = unsafe {
        libc::fdopendir(fd.into_raw_fd())
    };

    if dir.is_null() {
        Err(Error::last_os_error())
    } else {
        Ok(Dir{inner: dir})
    }
}
