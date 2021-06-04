use crate::Hash;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::Permissions;
use std::fs::create_dir;
use std::io::Error;
use std::io::ErrorKind::AlreadyExists;
use std::io::ErrorKind::NotFound;
use std::io::Read;
use std::io::Result;
use std::io::Seek;
use std::io::SeekFrom;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::path::PathBuf;
use wallace_fsutil as fsutil;

/// Handle to an opened volume.
pub struct Volume
{
    directory: File,
}

impl Volume
{
    /// Create a new volume at the given path,
    /// which must not yet exist.
    ///
    /// This does not open the volume.
    /// You can open the volume with [`Volume::open`].
    pub fn create(path: impl Into<PathBuf>) -> Result<()>
    {
        let mut pathbuf = path.into();

        create_dir(&pathbuf)?;

        pathbuf.push("stash");
        create_dir(&pathbuf)?;

        Ok(())
    }

    /// Open the volume at the given path,
    /// which must already be a volume
    /// created previously by [`Volume::create`].
    pub fn open(path: impl AsRef<Path>) -> Result<Self>
    {
        let directory =
            OpenOptions::new()
            .custom_flags(libc::O_DIRECTORY)
            .read(true)
            .open(path)?;
        Ok(Self{directory})
    }

    /// Like [`Volume::insert_from_path`], but given an already opened file.
    ///
    /// The path from which the file was opened, if any, is irrelevant.
    /// It could even be a file opened with `O_TMPFILE`,
    /// or a file that was sent over a Unix domain socket.
    pub fn insert_from_file(&self, mut file: File) -> Result<Hash>
    {
        // Verify that the file is a regular file.
        // If not, we cannot hard link it as an object.
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            // TODO: Return a more descriptive error.
            return Err(Error::from_raw_os_error(libc::EISDIR));
        }

        // We must seek the file to the beginning to start hashing it.
        // The file offset may be positioned anywhere prior to the call.
        file.seek(SeekFrom::Start(0))?;
        let hash = Hash::from_reader(&mut file)?;
        let path = format!("stash/{}", hash);

        // Unfortunately, the AT_EMPTY_PATH flag requires a special capability.
        // Fortunately, if /proc is available, we can apply this cute trick.
        // It is documented in the linkat(2) man page.
        let proc_path = format!("/proc/self/fd/{}", file.as_raw_fd());
        let linkat_result = fsutil::linkat(
            &libc::AT_FDCWD, proc_path, // old path
            &self.directory, path,      // new path
            libc::AT_SYMLINK_FOLLOW,    // see linkat(2)
        );

        // If the object already exists, then that is totally fine.
        // We will not touch this file anymore, and use the existing one.
        match linkat_result {
            Ok(()) => (),
            Err(err) if err.kind() == AlreadyExists => (),
            Err(err) => return Err(err),
        }

        // Make the file read-only to prevent tampering.
        // Not fool proof, as it can be chmodded again,
        // but that would be PEBKAC and not our problem.
        let readonly = Permissions::from_mode(0o400);
        file.set_permissions(readonly)?;

        Ok(hash)
    }

    /// Insert an object as a hard link to the file at the given path.
    ///
    /// The file is verified to be a regular file.
    /// Symbolic links, directories, and other file types are not supported.
    /// If this verification fails, then this function won’t do anything.
    ///
    /// The file must be readable, otherwise we cannot compute the hash.
    /// This function may change the permission bits of the file.
    /// If so, you should not change the permission bits of the file anymore.
    /// In fact, it would be best if you remove the path after,
    /// so as to avoid accidentally changing the file through it.
    ///
    /// This function assumes that the file contents
    /// remain untouched while the function is running.
    /// After the function returns successfully,
    /// the file contents should not be changed.
    /// Doing so would corrupt the volume.
    pub fn insert_from_path(&self, path: impl AsRef<Path>) -> Result<Hash>
    {
        // First we are going to open the file.
        // Then we will proceed as in insert_from_file.
        // It is amazing how little we have to do with the path.

        // We pass the following flags to the open syscall:
        let open_flags
            = libc::O_NOCTTY    // We don’t want a controlling terminal.
            | libc::O_NOFOLLOW  // We don’t want to follow symbolic links.
            | libc::O_CLOEXEC;  // We don’t want to keep the file open on exec.

        let file =
            OpenOptions::new()
            .read(true)
            .custom_flags(open_flags)
            .open(&path)?;

        self.insert_from_file(file)
    }

    /// Retrieve a read-only handle to an object,
    /// as well as the size of the object in bytes.
    ///
    /// If the object does not exist, this method returns [`None`].
    /// The returned reader/seeker is backed by a file,
    /// but it is not possible to recover the [`File`] interface.
    pub fn get(&self, hash: Hash) -> Result<Option<(impl Read + Seek, u64)>>
    {
        // Prevent any funny business from happening.
        // O_CLOEXEC:  Close the file if we spawn a subprocess.
        // O_NOCTTY:   Do not make any TTY become the controlling terminal.
        // O_NOFOLLOW: Do not follow any symbolic links.
        let open_flags = { use libc::*; O_RDONLY | O_CLOEXEC |
                                        O_NOCTTY | O_NOFOLLOW };

        // The mode is irrelevant because we are opening the file read-only.
        // However, we must still pass it to openat.
        let open_mode = 0;

        // Open the file backing the object.
        let path = format!("stash/{}", hash);
        let file_result = fsutil::openat(&self.directory, path,
                                         open_flags, open_mode);

        // If the file does not exist, return None.
        let file = match file_result {
            Ok(file) => file,
            Err(err) if err.kind() == NotFound =>
                return Ok(None),
            Err(err) => return Err(err),
        };

        // Find the size of the file.
        let metadata = file.metadata()?;
        let size = metadata.len();

        // Check that the file is regular.
        // If not, the volume is corrupt.
        if !metadata.is_file() {
            return Err(Error::from_raw_os_error(libc::EISDIR));
        }

        Ok(Some((file, size)))
    }
}

#[cfg(test)]
mod tests
{
    use std::env::temp_dir;
    use std::fs::remove_dir_all;
    use std::fs::write;
    use std::io::Cursor;
    use std::io::ErrorKind::AlreadyExists;
    use super::*;

    /// Create a directory for temporarily storing test data.
    /// The directory will be removed at the start of the test,
    /// which means it remains available for inspection after testing.
    fn make_temp_dir(name: impl AsRef<Path>) -> Result<PathBuf>
    {
        let mut parent = temp_dir();
        parent.push(name);
        drop(remove_dir_all(&parent));
        create_dir(&parent)?;
        Ok(parent)
    }

    #[test]
    fn test_create_exists()
    {
        let parent = make_temp_dir("test_create_exists").unwrap();

        // Create paths for to-be-existing paths.
        let mut path_reg = parent.clone();
        let mut path_dir = parent;
        path_reg.push("regular");
        path_dir.push("directory");

        // Create to-be-existing paths.
        File::create(&path_reg).unwrap();
        create_dir(&path_dir).unwrap();

        // Try to make volumes at existing paths.
        let result_reg = Volume::create(path_reg);
        let result_dir = Volume::create(path_dir);

        // Should fail with EEXIST.
        assert_eq!(result_reg.err().map(|e| e.kind()), Some(AlreadyExists));
        assert_eq!(result_dir.err().map(|e| e.kind()), Some(AlreadyExists));
    }

    #[test]
    fn test_insert_from_path_get()
    {
        let parent = make_temp_dir("test_insert_from_path_get").unwrap();

        // Create paths for test files and volume.
        let mut path_input1 = parent.clone();
        let mut path_input2 = parent.clone();
        let mut path_volume = parent;
        path_input1.push("input1");
        path_input2.push("input2");
        path_volume.push("volume");

        // Create and open the volume.
        Volume::create(&path_volume).unwrap();
        let volume = Volume::open(path_volume).unwrap();

        // Write test files.
        write(&path_input1, "hello").unwrap();
        write(&path_input2, "你好").unwrap();
        let hash1_object1 = volume.insert_from_path(&path_input1).unwrap();
        let hash1_object2 = volume.insert_from_path(&path_input2).unwrap();

        // Insert them again.
        let hash2_object1 = volume.insert_from_path(&path_input1).unwrap();
        let hash2_object2 = volume.insert_from_path(&path_input2).unwrap();

        // Check that the hashes were the same.
        assert_eq!(hash1_object1, hash2_object1);
        assert_eq!(hash1_object2, hash2_object2);

        // Check that we can retrieve the objects.
        let examples = &[(hash1_object1, "hello"),
                         (hash1_object2, "你好")];
        for &(hash, expected_data) in examples {
            let (mut read, actual_size) = volume.get(hash).unwrap().unwrap();
            let mut actual_data = Vec::new();
            read.read_to_end(&mut actual_data).unwrap();
            assert_eq!(actual_data, expected_data.as_bytes());
            assert_eq!(actual_size, expected_data.len() as u64);
        }

        // Check that getting a non-existing object succeeds.
        let mut cursor = Cursor::new(&mut []);
        let nonexistent = Hash::from_reader(&mut cursor).unwrap();
        let result = volume.get(nonexistent).unwrap();
        assert!(result.is_none());
    }
}
