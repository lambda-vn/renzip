//! Detection and parsing of the .rpa header line.
//!
//! Cross-checked against renpy/loader.py (function `index_archive` in the
//! official renpy/renpy repository, every tag from 6.18 to 8.5.3): for
//! RPA-3.0 the engine reads the key from a single field, `int(l[25:33], 16)`,
//! and nowhere folds several header words together. Folding every remaining
//! whitespace-separated hex word into the key by XOR is rpatool's behavior
//! (commit 5a3b12c, 2016), not the engine's; it is kept here anyway; a
//! single-subkey header still parses the same way, and it is what lets an
//! RPA-3.2 header (one extra field before the key) parse without a special
//! case beyond starting the fold one word later.

use crate::error::{Error, Result};

/// Archive format version, as announced by the header line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Version {
    /// Index lives in a separate .rpi file next to the archive.
    Rpa1_0,
    /// Plain offsets and lengths.
    Rpa2_0,
    /// Offsets and lengths obfuscated by XOR with a key.
    Rpa3_0,
    /// Same as 3.0 with one extra header field before the key.
    Rpa3_2,
}

impl Version {
    /// The version as it appears in a header line.
    pub fn as_str(self) -> &'static str {
        match self {
            Version::Rpa1_0 => "RPA-1.0",
            Version::Rpa2_0 => "RPA-2.0",
            Version::Rpa3_0 => "RPA-3.0",
            Version::Rpa3_2 => "RPA-3.2",
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Everything the header line says about where and how to read the index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// Announced format version.
    pub version: Version,
    /// Absolute offset of the compressed index inside the archive.
    pub index_offset: u64,
    /// XOR key applied to entry offsets and lengths, 0 when there is none.
    pub key: u64,
}

/// Longest header line this crate will look at.
///
/// A legitimate line is at most about 40 bytes; the cap keeps a headerless
/// file (an RPA-1.0 archive, or something that is not an archive at all) from
/// being scanned for a newline that may only appear megabytes later.
pub const MAX_HEADER_LEN: usize = 128;

/// Parse a header line, without its trailing newline.
///
/// # Errors
///
/// [`Error::UnsupportedHeader`] when the magic is unknown, a field is
/// missing, or a hexadecimal field does not parse.
pub fn parse_line(line: &str) -> Result<Header> {
    let unsupported = || Error::UnsupportedHeader(line.to_string());

    let mut parts = line.split_whitespace();
    let magic = parts.next().ok_or_else(unsupported)?;
    let fields: Vec<&str> = parts.collect();

    let (version, key_from) = match magic {
        "RPA-2.0" => (Version::Rpa2_0, None),
        "RPA-3.0" => (Version::Rpa3_0, Some(1)),
        "RPA-3.2" => (Version::Rpa3_2, Some(2)),
        _ => return Err(unsupported()),
    };

    let index_offset = u64::from_str_radix(fields.first().ok_or_else(unsupported)?, 16)
        .map_err(|_| unsupported())?;

    let mut key = 0u64;
    if let Some(from) = key_from {
        let subkeys = fields.get(from..).unwrap_or(&[]);
        if subkeys.is_empty() {
            return Err(unsupported());
        }
        for subkey in subkeys {
            key ^= u64::from_str_radix(subkey, 16).map_err(|_| unsupported())?;
        }
    }

    Ok(Header {
        version,
        index_offset,
        key,
    })
}

/// Render the header line, newline included, for a freshly built archive.
///
/// The width of both fields is fixed, which is what lets the builder write a
/// placeholder first and overwrite it with the real offset once the index has
/// been placed.
pub fn format_line(version: Version, index_offset: u64, key: u64) -> Result<String> {
    match version {
        Version::Rpa2_0 => Ok(format!("RPA-2.0 {index_offset:016x}\n")),
        Version::Rpa3_0 => Ok(format!("RPA-3.0 {index_offset:016x} {key:08x}\n")),
        other => Err(Error::UnsupportedWriteVersion(other.as_str())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rpa_2_0() {
        let header = parse_line("RPA-2.0 0000000000001234").unwrap();
        assert_eq!(header.version, Version::Rpa2_0);
        assert_eq!(header.index_offset, 0x1234);
        assert_eq!(header.key, 0);
    }

    #[test]
    fn parses_rpa_3_0_and_folds_every_subkey() {
        let header = parse_line("RPA-3.0 00000000000000ff deadbeef").unwrap();
        assert_eq!(header.version, Version::Rpa3_0);
        assert_eq!(header.index_offset, 0xff);
        assert_eq!(header.key, 0xdead_beef);

        let folded = parse_line("RPA-3.0 00000000000000ff 0000000f 000000f0").unwrap();
        assert_eq!(folded.key, 0xff);
    }

    #[test]
    fn rpa_3_2_skips_the_extra_field() {
        // "F" is the literal middle field seen in an actual RPA-3.2 archive
        // (Zeliria Sanctuary, Shizmob/rpatool#9), not a placeholder.
        let header = parse_line("RPA-3.2 0000000000000010 F deadbeef").unwrap();
        assert_eq!(header.version, Version::Rpa3_2);
        assert_eq!(header.index_offset, 0x10);
        assert_eq!(header.key, 0xdead_beef);
    }

    #[test]
    fn rejects_unknown_or_incomplete_headers() {
        assert!(parse_line("PK\u{3}\u{4}").is_err());
        assert!(parse_line("RPA-3.0 0000000000000010").is_err());
        assert!(parse_line("RPA-2.0").is_err());
        assert!(parse_line("RPA-2.0 zzzz").is_err());
    }

    #[test]
    fn format_line_round_trips() {
        let line = format_line(Version::Rpa3_0, 0x2a, 0xdead_beef).unwrap();
        assert_eq!(line, "RPA-3.0 000000000000002a deadbeef\n");
        let parsed = parse_line(line.trim_end()).unwrap();
        assert_eq!(parsed.index_offset, 0x2a);
        assert_eq!(parsed.key, 0xdead_beef);

        assert!(format_line(Version::Rpa3_2, 0, 0).is_err());
    }
}
