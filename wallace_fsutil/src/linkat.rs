use std::ffi::CString;
use std::io::Error;
use std::io::Result;
use std::os::raw::c_int;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

/// Perform the `linkat` system call.
pub fn linkat(
    olddir: &impl AsRawFd,
    oldpath: impl AsRef<Path>,
    newdir: &impl AsRawFd,
    newpath: impl AsRef<Path>,
    flags: c_int,
) -> Result<()>
{
    let cstr = |p: &Path| CString::new(p.as_os_str().as_bytes());

    let oldpath_c: CString = cstr(oldpath.as_ref())?;
    let newpath_c: CString = cstr(newpath.as_ref())?;

    // SAFETY: All C strings are of type CString
    // and are therefore null-terminated.
    let status = unsafe {
        libc::linkat(
            olddir.as_raw_fd(),
            oldpath_c.as_ptr(),
            newdir.as_raw_fd(),
            newpath_c.as_ptr(),
            flags,
        )
    };

    if status == -1 {
        Err(Error::last_os_error())
    } else {
        Ok(())
    }
}
