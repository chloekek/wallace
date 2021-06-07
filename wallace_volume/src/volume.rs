use crate::Hash;
use crate::InvalidHash;
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
use std::io::copy;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::path::PathBuf;
use wallace_fsutil as fsutil;

/// Handle to an opened volume.
///
/// See the [crate documentation](index.html)
/// for more information on what volumes are.
///
/// The volume handle is backed by a file descriptor,
/// and it will continue to be usable just fine
/// if the path to the volume’s directory changes
/// (although this is a rather obscure use case).
pub struct Volume
{
    directory: File,
}

impl Volume
{
    /// Create a new volume at the given path,
    /// which must not yet exist.
    ///
    /// The volume starts out with no objects stored in it.
    /// You can open the volume with [`Volume::open`].
    pub fn create(path: impl Into<PathBuf>) -> Result<()>
    {
        let mut pathbuf = path.into();

        create_dir(&pathbuf)?;

        pathbuf.push("objects");
        create_dir(&pathbuf)?;

        Ok(())
    }

    /// Open the volume at the given path,
    /// which must already be created previously
    /// using the [`Volume::create`] method.
    pub fn open(path: impl AsRef<Path>) -> Result<Self>
    {
        let directory =
            OpenOptions::new()
            .custom_flags(libc::O_DIRECTORY)
            .read(true)
            .open(path)?;
        Ok(Self{directory})
    }

    /// Insert an object into the volume by
    /// creating a hard link to a given file.
    ///
    /// This method will read the file to determine the hash of the object.
    /// It will then create the appropriate hard link in the volume.
    /// Finally, it will make the file read-only using `chmod`.
    ///
    /// The path from which the file was opened, if any, is irrelevant.
    /// It could even be a file opened with `O_TMPFILE`,
    /// or a file that was sent over a Unix domain socket.
    ///
    /// # Preconditions
    ///
    /// This method has a few preconditions, which must all hold:
    ///
    ///  - The file must be a regular file.
    ///  - The file must be owned by the caller.
    ///  - The file must be readable.
    ///
    /// Violating any precondition will cause this method to return an error,
    /// and leave the volume in the state it was prior to calling this method.
    ///
    /// # Special considerations
    ///
    /// It is paramount that the file will not be modified
    /// after this method starts to do its work.
    /// Such modifications may result in a corrupted volume.
    /// This includes modifications to any existing hard links.
    ///
    /// If the object already exists in the volume,
    /// the existing file is retained, and the given file is ignored.
    /// However, the given file will still be read to compute its hash.
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
        let hash = Hash::compute_from_reader(&mut file)?;
        let path = format!("objects/{}", hash);

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

    /// Drain the given reader into a temporary file,
    /// and proceed as in [`Volume::insert_from_file`].
    pub fn insert_from_reader(&self, reader: &mut impl Read) -> Result<Hash>
    {
        // By using O_TMPFILE, Linux will create a file with no path.
        // We can then write this file and pass it to insert_from_file.
        let open_flags = libc::O_RDWR | libc::O_TMPFILE;

        // We must write the file and then read it,
        // so we will use this open mode.
        let open_mode = 0o600;

        // We must still pass some path to openat.
        // Linux uses this to determine the file system
        // on which the file is to be stored.
        // We pass the path to the volume directory.
        let mut tmpfile = fsutil::openat(&self.directory, ".",
                                         open_flags, open_mode)?;

        // Drain the entire reader into the temporary file.
        copy(reader, &mut tmpfile)?;

        self.insert_from_file(tmpfile)
    }

    /// Open the file at the given path,
    /// and proceed as in [`Volume::insert_from_file`].
    ///
    /// If the file is a fifo, this method won’t block waiting for a writer.
    /// It will immediately return an error because fifos are not regular files.
    /// Similar shenanigans with other exotic file types are also avoided.
    pub fn insert_from_path(&self, path: impl AsRef<Path>) -> Result<Hash>
    {
        // First we are going to open the file.
        // Then we will proceed as in insert_from_file.
        // It is amazing how little we have to do with the path.

        // We pass the following flags to the open syscall:
        let open_flags
            = libc::O_NOCTTY    // We don’t want a controlling terminal.
            | libc::O_NOFOLLOW  // We don’t want to follow symbolic links.
            | libc::O_CLOEXEC   // We don’t want to keep the file open on exec.
            | libc::O_NONBLOCK; // Don’t block if the file is a fifo.

        let file =
            OpenOptions::new()
            .read(true)
            .custom_flags(open_flags)
            .open(&path)?;

        // Switch the file back to blocking mode.
        // We only needed non-blocking mode to
        // open a potential fifo without blocking.
        let fd_flags = fsutil::fcntl_getfd(&file)?;
        fsutil::fcntl_setfd(&file, fd_flags & !libc::O_NONBLOCK)?;

        self.insert_from_file(file)
    }

    /// Retrieve a read-only handle to an object’s byte array,
    /// as well as the size of the object in bytes.
    ///
    /// If the object does not exist, this method returns [`None`].
    /// The returned reader/seeker is backed by a file,
    /// but this fact is hidden using impl trait
    /// because the file should not be modified.
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
        let path = format!("objects/{}", hash);
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

    /// Return an iterator over the objects in the volume.
    ///
    /// This iterator will not open the objects,
    /// it will only yield their hashes.
    pub fn all(&self) -> Result<impl Iterator<Item=Result<Hash>>>
    {
        struct All
        {
            objects_dir: fsutil::Dir,
        }

        impl Iterator for All
        {
            type Item = Result<Hash>;

            fn next(&mut self) -> Option<Self::Item>
            {
                match fsutil::readdir(&mut self.objects_dir) {
                    Err(err) => Some(Err(err)),
                    Ok(None) => None,
                    Ok(Some(dirent)) => {
                        let filename = dirent.d_name().to_bytes();
                        match Hash::from_ascii(filename) {
                            Ok(hash) => Some(Ok(hash)),
                            Err(InvalidHash) => self.next(),
                        }
                    },
                }
            }
        }

        let open_flags = libc::O_RDONLY | libc::O_DIRECTORY;
        let objects_directory =
            fsutil::openat(&self.directory, "objects", open_flags, 0)?;
        let objects_dir = fsutil::fdopendir(objects_directory)?;
        Ok(All{objects_dir})
    }
}

#[cfg(test)]
mod tests
{
    use crate::TestData;
    use std::io::Cursor;
    use std::io::ErrorKind::AlreadyExists;
    use super::*;

    #[test]
    fn test_create_exists()
    {
        let test_data = TestData::new("test_create_exists").unwrap();
        let result1 = Volume::create(test_data.volume1_path);
        let result2 = Volume::create(test_data.regular1_path);
        assert_eq!(result1.err().map(|e| e.kind()), Some(AlreadyExists));
        assert_eq!(result2.err().map(|e| e.kind()), Some(AlreadyExists));
    }

    #[test]
    fn test_insert_from_path_non_reg()
    {
        // Prepare the test.
        let test_data = TestData::new("test_insert_from_path_non_reg").unwrap();
        let volume = Volume::open(test_data.volume1_path).unwrap();

        // Check that objects cannot be inserted.
        assert!(volume.insert_from_path(&test_data.character1_path) .is_err());
        assert!(volume.insert_from_path(&test_data.directory1_path) .is_err());
        assert!(volume.insert_from_path(&test_data.fifo1_path).is_err());
        assert!(volume.insert_from_path(&test_data.socket1_path) .is_err());
        assert!(volume.insert_from_path(&test_data.symlink1_path).is_err());
    }

    #[test]
    fn test_insert_from_reader()
    {
        // Prepare the test.
        let test_data = TestData::new("test_insert_from_reader").unwrap();
        let volume = Volume::open(test_data.volume1_path).unwrap();

        // Insert the object.
        let mut cursor = Cursor::new(&test_data.regular1_contents);
        let hash = volume.insert_from_reader(&mut cursor).unwrap();

        // Get the object.
        let (mut read, size) = volume.get(hash).unwrap().unwrap();
        let mut data = Vec::new();
        read.read_to_end(&mut data).unwrap();

        // Check the results.
        assert_eq!(hash, test_data.regular1_hash);
        assert_eq!(data, test_data.regular1_contents);
        assert_eq!(size, test_data.regular1_contents.len() as u64);
    }

    #[test]
    fn test_insert_from_path()
    {
        // Prepare the test.
        let test_data = TestData::new("test_insert_from_path").unwrap();
        let volume = Volume::open(test_data.volume1_path).unwrap();

        // Insert the objects.
        let hash1 = volume.insert_from_path(&test_data.regular1_path).unwrap();
        let hash2 = volume.insert_from_path(&test_data.regular2_path).unwrap();

        // Get the objects.
        let (mut read1, size1) = volume.get(hash1).unwrap().unwrap();
        let (mut read2, size2) = volume.get(hash2).unwrap().unwrap();
        let mut data1 = Vec::new();
        let mut data2 = Vec::new();
        read1.read_to_end(&mut data1).unwrap();
        read2.read_to_end(&mut data2).unwrap();

        // Check the results.
        assert_eq!(hash1, test_data.regular1_hash);
        assert_eq!(hash2, test_data.regular2_hash);
        assert_eq!(data1, test_data.regular1_contents);
        assert_eq!(data2, test_data.regular2_contents);
        assert_eq!(size1, test_data.regular1_contents.len() as u64);
        assert_eq!(size2, test_data.regular2_contents.len() as u64);
    }

    #[test]
    fn test_all()
    {
        // Prepare the test.
        let test_data = TestData::new("test_all").unwrap();
        let volume = Volume::open(test_data.volume1_path).unwrap();

        // Insert the objects.
        let hash1 = volume.insert_from_path(&test_data.regular1_path).unwrap();
        let hash2 = volume.insert_from_path(&test_data.regular2_path).unwrap();

        // List the objects.
        let all = volume.all().unwrap();
        let mut actual = all.collect::<Result<Vec<_>>>().unwrap();

        // Check the results..
        let mut expected = [hash1, hash2];
        expected.sort_by_key(|h| h.bytes);
        actual.sort_by_key(|h| h.bytes);
        assert_eq!(actual, expected);
    }
}
