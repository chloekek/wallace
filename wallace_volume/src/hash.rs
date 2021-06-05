use std::fmt;
use std::io;
use std::str::FromStr;
use wallace_sha256::Sha256;

/// Hash of an object used to uniquely identify it.
///
/// For more information about hashes,
/// refer to the section about
/// content addressable storage
/// in the crate documentation.
///
/// The [`Display`][`fmt::Display`] impl formats the hash
/// as a 64-digit lowercase hexadecimal number.
/// The [`FromStr`] impl parses this same format.
/// This hexadecimal format is used consistently
/// when hashes need to be communicated as text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Hash
{
    /// The bytes that make up the hash.
    /// These are _not_ the bytes that the hash was computed from;
    /// those bytes cannot be recovered from the hash alone.
    /// You typically do not need to access this field.
    pub bytes: [u8; 32],
}

impl Hash
{
    /// Read all bytes from the reader and compute their hash.
    pub fn from_reader(r: &mut impl io::Read) -> io::Result<Self>
    {
        let mut sha256 = Sha256::new();
        io::copy(r, &mut sha256)?;
        let bytes = sha256.finalize();
        Ok(Self{bytes})
    }
}

impl fmt::Display for Hash
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        for byte in &self.bytes {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// Returned when a hash could not be parsed.
#[derive(Clone, Copy, Debug)]
pub struct InvalidHash;

impl FromStr for Hash
{
    type Err = InvalidHash;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        if s.len() != 64 {
            return Err(InvalidHash);
        }

        fn hex(c: u8) -> Result<u8, InvalidHash>
        {
            match c {
                b'0' ..= b'9' => Ok(c - b'0'),
                b'a' ..= b'f' => Ok(c - b'a' + 10),
                _ => Err(InvalidHash),
            }
        }

        let mut bytes = [0; 32];
        for (i, pair) in s.as_bytes().chunks(2).enumerate() {
            bytes[i] = hex(pair[0])? << 4 | hex(pair[1])?;
        }

        Ok(Self{bytes})
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_from_reader()
    {
        let table: &[(&[_], _)] = &[
            (b"",
             concat!("e3b0c44298fc1c149afbf4c8996fb924",
                     "27ae41e4649b934ca495991b7852b855")),
            (b"Hello, world!",
             concat!("315f5bdb76d078c43b8ac0064e4a0164",
                     "612b1fce77c869345bfc94c75894edd3")),
        ];
        for &(input, expected) in table {
            let mut cursor = Cursor::new(input);
            let actual_hash = Hash::from_reader(&mut cursor).unwrap();
            let actual = format!("{}", actual_hash);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_from_str()
    {
        let examples = &[

            // Parseable examples.
            (true, concat!("00000000000000000000000000000000",
                           "00000000000000000000000000000000")),
            (true, concat!("ffffffffffffffffffffffffffffffff",
                           "ffffffffffffffffffffffffffffffff")),
            (true, concat!("e3b0c44298fc1c149afbf4c8996fb924",
                           "27ae41e4649b934ca495991b7852b855")),

            // Unparseable examples.
            (false, concat!("")),
            (false, concat!("E3B0C44298FC1C149AFBF4C8996FB924",
                            "27AE41E4649B934CA495991B7852B855")),
            (false, concat!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
                            "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX")),

        ];

        for &(expected, example) in examples {
            if expected {
                assert_eq!(
                    Hash::from_str(example)
                        .map(|h| format!("{}", h))
                        .ok(),
                    Some(example.to_string()),
                );
            } else {
                assert!(Hash::from_str(example).is_err());
            }
        }
    }
}
