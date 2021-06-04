use std::io::Error;
use std::io::Result;
use std::os::raw::c_int;
use std::os::unix::io::AsRawFd;

/// Perform the `fcntl` system call with command `F_GETFD`.
pub fn fcntl_getfd(fd: &impl AsRawFd) -> Result<c_int>
{
    // SAFETY: This usage is safe.
    let status = unsafe {
        libc::fcntl(fd.as_raw_fd(), libc::F_GETFD)
    };

    if status == -1 {
        Err(Error::last_os_error())
    } else {
        Ok(status)
    }
}

/// Perform the `fcntl` system call with command `F_SETFD`.
pub fn fcntl_setfd(fd: &impl AsRawFd, arg: c_int) -> Result<()>
{
    // SAFETY: This usage is safe.
    let status = unsafe {
        libc::fcntl(fd.as_raw_fd(), libc::F_SETFD, arg)
    };

    if status == -1 {
        Err(Error::last_os_error())
    } else {
        Ok(())
    }
}
