use libc::mode_t;
use std::ffi::CString;
use std::fs::File;
use std::io::Error;
use std::io::Result;
use std::os::raw::c_int;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
use std::path::Path;

/// Perform the `openat` system call.
pub fn openat(
    dir: &impl AsRawFd,
    pathname: impl AsRef<Path>,
    flags: c_int,
    mode: mode_t,
) -> Result<File>
{
    let cstr = |p: &Path| CString::new(p.as_os_str().as_bytes());

    let pathname_c = cstr(pathname.as_ref())?;

    // SAFETY: The C string is of type CString
    // and is therefore null-terminated.
    let fd = unsafe {
        libc::openat(
            dir.as_raw_fd(),
            pathname_c.as_ptr(),
            flags,
            mode,
        )
    };

    if fd == -1 {
        Err(Error::last_os_error())
    } else {
        // SAFETY: FromRawFd::from_raw_fd being unsafe is silly.
        Ok(unsafe { File::from_raw_fd(fd) })
    }
}
