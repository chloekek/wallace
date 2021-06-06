use std::str::FromStr;
use wallace_volume::Hash;

/// Abstract syntax tree for paths that
/// address objects or collections of objects.
///
/// The [`FromStr`] impl takes [`str`],
/// rather than [`Path`][`std::path::Path`],
/// as our paths are always encoded as UTF-8,
/// and always use forward solidi as path separators.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParsedPath
{
    /// Path to the root directory.
    Root,

    /// Path to the objects directory.
    Objects,

    /// Path to an object in the objects directory.
    ObjectsObject(Hash),
}

impl ParsedPath
{
    /// Parse a path from its sequence of components.
    ///
    /// Path components are the strings between the forward solidi.
    /// This function ignores empty components.
    pub fn from_components<'a>(components: impl IntoIterator<Item=&'a str>)
        -> Option<Self>
    {
        // Keep only non-empty components.
        let components = components.into_iter();
        let mut components = components.filter(|&c| c != "");

        match components.next() {
            None            => Some(Self::Root),
            Some("objects") => Self::from_objects_components(components),
            _               => None,
        }
    }

    fn from_objects_components<'a>(mut components: impl Iterator<Item=&'a str>)
        -> Option<Self>
    {
        match (components.next(), components.next()) {
            (None,       _      ) => Some(Self::Objects),
            (Some(hash), None   ) => hash.parse().ok().map(Self::ObjectsObject),
            (Some(_),    Some(_)) => None,
        }
    }
}

/// Returned when a path could not be parsed.
#[derive(Clone, Copy, Debug)]
pub struct InvalidPath;

impl FromStr for ParsedPath
{
    type Err = InvalidPath;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        Self::from_components(s.split('/'))
            .ok_or(InvalidPath)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_from_str()
    {
        let examples = &[

            ("", Some(ParsedPath::Root)),
            ("/", Some(ParsedPath::Root)),
            ("//", Some(ParsedPath::Root)),

            ("objects", Some(ParsedPath::Objects)),
            ("objects/", Some(ParsedPath::Objects)),
            ("/objects", Some(ParsedPath::Objects)),
            ("/objects/", Some(ParsedPath::Objects)),

            (concat!("/objects/ffffffffffffffffffffffffffffffff",
                              "ffffffffffffffffffffffffffffffff"),
             Some(ParsedPath::ObjectsObject(Hash{bytes: [0xFF; 32]}))),
            (concat!("/objects/ffffffffffffffffffffffffffffffff",
                              "ffffffffffffffffffffffffffffffff/"),
             Some(ParsedPath::ObjectsObject(Hash{bytes: [0xFF; 32]}))),

            ("hello", None),
            ("/hello", None),
            ("objectsx", None),
            ("/objectsx", None),
            ("/objects/x", None),
            (concat!("/objects/xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                              "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"), None),
            (concat!("/objects/FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
                              "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"), None),
            (concat!("/objects/00000000000000000000000000000000",
                              "000000000000000000000000000000000"), None),

        ];

        for (input, expected) in examples {
            let actual = input.parse().ok();
            assert_eq!(&actual, expected);
        }
    }
}
