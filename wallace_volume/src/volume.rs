use crate::Hash;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::Permissions;
use std::fs::create_dir;
use std::io::Error;
use std::io::ErrorKind::NotFound;
use std::io::Read;
use std::io::Result;
use std::io::Seek;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
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

    /// Insert the contents of a given file as an object into the volume,
    /// and remove the file from the given source path.
    ///
    /// This function assumes that the file contents
    /// remain untouched while the function is running.
    /// After the function returns successfully,
    /// the file contents should not be changed.
    /// Doing so would corrupt the volume.
    ///
    /// The insertion procedure operates as follows:
    ///
    ///  1. The source file is opened in read-only mode.
    ///  2. Some sanity checks on the source file are performed.
    ///  3. The cryptographic hash of the source file is computed.
    ///  4. This hash is used to compute the destination path.
    ///  5. The source path is renamed to the destination path.
    ///  6. The source file is made read-only.
    ///  7. The hash is returned, to be used as the identifier of the object.
    pub fn insert_from_file(&self, source_path: impl AsRef<Path>)
        -> Result<Hash>
    {
        // Prevent any funny business from happening.
        // O_NOCTTY:   Do not make any TTY become the controlling terminal.
        // O_NOFOLLOW: Do not follow any symbolic links.
        let open_flags = libc::O_NOCTTY | libc::O_NOFOLLOW;

        // Open the source file.
        let mut file =
            OpenOptions::new()
            .read(true)
            .custom_flags(open_flags)
            .open(&source_path)?;

        // Check that the source file is regular.
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(Error::from_raw_os_error(libc::EISDIR));
        }

        // Compute the hash of the source file.
        let hash = Hash::from_reader(&mut file)?;

        // Rename the source path to the destination path.
        let destination_path = format!("stash/{}", hash);
        fsutil::renameat(
            &libc::AT_FDCWD, source_path,
            &self.directory, destination_path,
        )?;

        // Set the file permissions to read-only.
        file.set_permissions(Permissions::from_mode(0o400))?;

        Ok(hash)
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
    fn test_insert_from_file_get()
    {
        let parent = make_temp_dir("test_insert_get").unwrap();

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
        let hash1_object1 = volume.insert_from_file(&path_input1).unwrap();
        let hash1_object2 = volume.insert_from_file(&path_input2).unwrap();

        // Insert them again.
        write(&path_input1, "hello").unwrap();
        write(&path_input2, "你好").unwrap();
        let hash2_object1 = volume.insert_from_file(&path_input1).unwrap();
        let hash2_object2 = volume.insert_from_file(&path_input2).unwrap();

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
