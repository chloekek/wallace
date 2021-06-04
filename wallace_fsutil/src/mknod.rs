use libc::dev_t;
use libc::mode_t;
use std::ffi::CString;
use std::io::Error;
use std::io::Result;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// Perform the `mknod` system call.
pub fn mknod(pathname: impl AsRef<Path>, mode: mode_t, dev: dev_t) -> Result<()>
{
    let cstr = |p: &Path| CString::new(p.as_os_str().as_bytes());

    let pathname_c = cstr(pathname.as_ref())?;

    // SAFETY: The C string is of type CString
    // and is therefore null-terminated.
    let fd = unsafe {
        libc::mknod(
            pathname_c.as_ptr(),
            mode,
            dev,
        )
    };

    if fd == -1 {
        Err(Error::last_os_error())
    } else {
        Ok(())
    }
}
