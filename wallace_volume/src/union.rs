use crate::Hash;
use crate::Volume;
use std::io::Read;
use std::io::Result;
use std::io::Seek;
use wallace_iterutil::iter_result_iter;

/// Retrieve a read-only handle to an object’s byte array,
/// as well as the size of the object in bytes,
/// from the first given volume that has it.
pub fn union_get<'a, I>(volumes: I, hash: Hash)
    -> Result<Option<(impl Read + Seek, u64)>>
    where I: IntoIterator<Item=&'a Volume>
{
    volumes
        .into_iter  ()
        .map        (|v| v.get(hash))
        .filter_map (|r| r.transpose())
        .next       ()
        .transpose  ()
}

/// Return an iterator over the objects in all the given volumes.
///
/// This iterator will not open the objects,
/// it will only yield their hashes.
pub fn union_all<'a, I>(volumes: I)
    -> impl 'a + Iterator<Item=Result<Hash>>
    where I: IntoIterator<Item=&'a Volume>
        , I::IntoIter: 'a
{
    volumes
        .into_iter ()
        .map       (|v| v.all())
        .flat_map  (|i| iter_result_iter(i))
        .map       (|r| r.unwrap_or_else(Err))
}

#[cfg(test)]
mod tests
{
    use crate::TestData;
    use super::*;

    #[test]
    fn test_union_get()
    {
        // Prepare the test.
        let test_data = TestData::new("test_union_get").unwrap();
        let volume1 = Volume::open(&test_data.volume1_path).unwrap();
        let volume2 = Volume::open(&test_data.volume2_path).unwrap();
        let volumes = || vec![&volume1, &volume2];

        // Insert the objects.
        let hash1 = volume1.insert_from_path(&test_data.regular1_path).unwrap();
        let hash2 = volume2.insert_from_path(&test_data.regular2_path).unwrap();

        // Get the objects.
        let (mut read1, size1) = union_get(volumes(), hash1).unwrap().unwrap();
        let (mut read2, size2) = union_get(volumes(), hash2).unwrap().unwrap();
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
    fn test_union_all()
    {
        // Prepare the test.
        let test_data = TestData::new("test_union_all").unwrap();
        let volume1 = Volume::open(&test_data.volume1_path).unwrap();
        let volume2 = Volume::open(&test_data.volume2_path).unwrap();
        let volumes = || vec![&volume1, &volume2];

        // Insert the objects.
        let hash1 = volume1.insert_from_path(&test_data.regular1_path).unwrap();
        let hash2 = volume2.insert_from_path(&test_data.regular2_path).unwrap();

        // List the objects.
        let mut actual =
            union_all(volumes())
            .collect::<Result<Vec<_>>>()
            .unwrap();

        // Check the results.
        let mut expected = [hash1, hash2];
        expected.sort_by_key(|h| h.bytes);
        actual.sort_by_key(|h| h.bytes);
        assert_eq!(actual, expected);
    }
}
