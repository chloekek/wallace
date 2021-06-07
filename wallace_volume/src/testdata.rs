use crate::Hash;
use crate::Volume;
use std::env;
use std::fs;
use std::io::Result;
use std::os::unix;
use std::path::PathBuf;
use wallace_fsutil as fsutil;

pub struct TestData
{
    pub root_path: PathBuf,

    pub volume1_path: PathBuf,
    pub volume2_path: PathBuf,

    pub regular1_path: PathBuf,
    pub regular2_path: PathBuf,

    pub regular1_contents: Vec<u8>,
    pub regular2_contents: Vec<u8>,

    pub regular1_hash: Hash,
    pub regular2_hash: Hash,

    pub character1_path: PathBuf,
    pub directory1_path: PathBuf,
    pub fifo1_path: PathBuf,
    pub socket1_path: PathBuf,
    pub symlink1_path: PathBuf,
}

impl TestData
{
    pub fn new(name: &str) -> Result<Self>
    {
        // Create root.
        let root_path = env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&root_path);
        fs::create_dir(&root_path)?;

        // Create volumes.
        let volume1_path = root_path.join("volume1");
        let volume2_path = root_path.join("volume2");
        Volume::create(&volume1_path)?;
        Volume::create(&volume2_path)?;

        // Create regular files.
        let regular1_path = root_path.join("regular1");
        let regular2_path = root_path.join("regular2");
        let regular1_contents = "hello".as_bytes().to_vec();
        let regular2_contents = "你好".as_bytes().to_vec();
        let regular1_hash = Hash::compute_from_bytes(&regular1_contents);
        let regular2_hash = Hash::compute_from_bytes(&regular2_contents);
        fs::write(&regular1_path, &regular1_contents)?;
        fs::write(&regular2_path, &regular2_contents)?;

        // Create special files.
        let character1_path = PathBuf::from("/dev/null");
        let directory1_path = root_path.join("directory1");
        let fifo1_path      = root_path.join("fifo1");
        let socket1_path    = root_path.join("socket1");
        let symlink1_path   = root_path.join("symlink1");
        fs::create_dir(&directory1_path)?;
        fsutil::mknod(&fifo1_path, libc::S_IFIFO | 0o644, 0)?;
        fsutil::mknod(&socket1_path, libc::S_IFSOCK | 0o644, 0)?;
        unix::fs::symlink("/etc/passwd", &symlink1_path)?;

        Ok(
            Self{
                root_path,
                volume1_path,
                volume2_path,
                regular1_path,
                regular2_path,
                regular1_contents,
                regular2_contents,
                regular1_hash,
                regular2_hash,
                character1_path,
                directory1_path,
                fifo1_path,
                socket1_path,
                symlink1_path,
            }
        )
    }
}
