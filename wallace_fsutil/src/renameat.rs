use std::ffi::CString;
use std::io::Error;
use std::io::Result;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

/// Perform the `renameat` system call.
pub fn renameat(
    from_dir:  &impl AsRawFd,
    from_path: impl AsRef<Path>,
    to_dir:    &impl AsRawFd,
    to_path:   impl AsRef<Path>,
) -> Result<()>
{
    let cstr = |p: &Path| CString::new(p.as_os_str().as_bytes());

    let from_path_c: CString = cstr(from_path.as_ref())?;
    let to_path_c:   CString = cstr(to_path.as_ref())?;

    // SAFETY: All C strings are of type CString
    // and are therefore null-terminated.
    let status = unsafe {
        libc::renameat(
            from_dir.as_raw_fd(),
            from_path_c.as_ptr(),
            to_dir.as_raw_fd(),
            to_path_c.as_ptr(),
        )
    };

    if status == -1 {
        Err(Error::last_os_error())
    } else {
        Ok(())
    }
}
