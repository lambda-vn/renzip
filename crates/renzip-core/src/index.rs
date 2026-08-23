//! Decoding and encoding of the archive index: zlib over a Python pickle.
//!
//! The index is the only compressed part of an .rpa; file data is stored raw.
//! Cross-checked against renpy/loader.py: the engine reads from the header
//! offset to the end of the file, inflates it, unpickles a dict mapping a
//! name to a list of tuples, and only ever looks at the first tuple of the
//! list. A tuple holds (offset, length) or (offset, length, prefix); for
//! RPA-3.x, offset and length are XORed with the header key while prefix is
//! not.
//!
//! Unpickling here never runs code: serde-pickle turns opcodes into plain
//! values, and a GLOBAL opcode is at most a name it recognizes, never a
//! callable it invokes. Anything that is not a dict of strings to lists of
//! integer tuples is a format error, not a case to be clever about.

use std::collections::BTreeMap;
use std::io::Read;

use flate2::read::ZlibDecoder;
use serde_pickle::{DeOptions, HashableValue, SerOptions, Value};

use crate::error::{Error, Result};

/// Largest index this crate will inflate, as a guard against a zlib bomb.
///
/// An index for 50 000 entries weighs a few megabytes once inflated, so the
/// cap sits far above any legitimate archive while still bounding what a
/// forged one can make us allocate.
pub const MAX_INDEX_BYTES: u64 = 256 * 1024 * 1024;

/// One entry as the index describes it, before any path or bounds check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEntry {
    /// Entry path, exactly as stored in the archive.
    pub path: String,
    /// Offset of the entry data inside the archive, key already applied.
    pub offset: u64,
    /// Total size of the entry, prefix included, key already applied.
    pub length: u64,
    /// Bytes to prepend to the data read at `offset`.
    pub prefix: Vec<u8>,
    /// How many tuples the index listed for this path.
    ///
    /// Ren'Py only ever reads the first one and so does this crate, but an
    /// archive with more is unusual enough to be worth reporting rather than
    /// swallowing.
    pub segment_count: usize,
}

/// Inflate and unpickle an index, applying `key` to offsets and lengths.
///
/// # Errors
///
/// [`Error::CorruptIndex`] when the stream does not inflate, does not
/// unpickle, exceeds [`MAX_INDEX_BYTES`], or does not have the shape
/// described above.
pub fn decode(compressed: impl Read, key: u64) -> Result<Vec<RawEntry>> {
    let mut raw = Vec::new();
    // take() one byte past the cap so that hitting it exactly is detectable.
    ZlibDecoder::new(compressed)
        .take(MAX_INDEX_BYTES + 1)
        .read_to_end(&mut raw)
        .map_err(|e| Error::CorruptIndex(format!("cannot inflate the index: {e}")))?;
    if raw.len() as u64 > MAX_INDEX_BYTES {
        return Err(Error::CorruptIndex(format!(
            "index inflates past the {MAX_INDEX_BYTES} byte limit"
        )));
    }

    // Globals are replaced by None instead of aborting the parse, because a
    // legitimate index contains one: Python pickles an empty bytes object at
    // protocol 2 as __builtin__.bytes(), which serde-pickle does not model
    // (it does model the _codecs.encode form used for non-empty ones). The
    // option only ever turns an object into None, never into a call, and a
    // None still has to pass the shape checks below.
    let options = DeOptions::new().replace_unresolved_globals();
    let value = serde_pickle::value_from_slice(&raw, options)
        .map_err(|e| Error::CorruptIndex(format!("cannot unpickle the index: {e}")))?;

    let Value::Dict(dict) = value else {
        return Err(Error::CorruptIndex("index is not a dict".into()));
    };

    let mut entries = Vec::with_capacity(dict.len());
    for (name, tuples) in dict {
        let path = decode_name(name)?;
        entries.push(decode_entry(path, tuples, key)?);
    }
    Ok(entries)
}

/// Turn an index key into a path string.
///
/// Archives written by a Python 2 era Ren'Py store names as byte strings; the
/// engine decodes those as latin-1, so a name that is not valid UTF-8 is
/// decoded the same way here rather than rejected.
fn decode_name(name: HashableValue) -> Result<String> {
    match name {
        HashableValue::String(s) => Ok(s),
        HashableValue::Bytes(b) => Ok(match String::from_utf8(b) {
            Ok(s) => s,
            Err(e) => e.into_bytes().iter().map(|&c| c as char).collect(),
        }),
        other => Err(Error::CorruptIndex(format!(
            "index key is not a string: {other:?}"
        ))),
    }
}

/// Decode the tuple list attached to one name.
fn decode_entry(path: String, tuples: Value, key: u64) -> Result<RawEntry> {
    let Value::List(items) = tuples else {
        return Err(Error::CorruptIndex(format!(
            "entry '{path}' is not a list of tuples"
        )));
    };
    let segment_count = items.len();
    let first = items
        .into_iter()
        .next()
        .ok_or_else(|| Error::CorruptIndex(format!("entry '{path}' has no tuple")))?;

    let Value::Tuple(fields) = first else {
        return Err(Error::CorruptIndex(format!(
            "entry '{path}' is not described by a tuple"
        )));
    };
    if fields.len() != 2 && fields.len() != 3 {
        return Err(Error::CorruptIndex(format!(
            "entry '{path}' has {} fields, expected 2 or 3",
            fields.len()
        )));
    }

    let offset = decode_int(&fields[0], &path)? ^ key;
    let length = decode_int(&fields[1], &path)? ^ key;
    let prefix = match fields.get(2) {
        None => Vec::new(),
        Some(Value::Bytes(b)) => b.clone(),
        // An empty bytes object written at protocol 2, once its unmodelled
        // constructor has been replaced by None. Ren'Py stores exactly that
        // for every entry it packs without a prefix.
        Some(Value::None) => Vec::new(),
        // A prefix pickled by Python 2 arrives as a byte string too, but one
        // pickled from a unicode object arrives as a String; Ren'Py writes it
        // back out as latin-1, so that is how it is turned into bytes here.
        Some(Value::String(s)) => s.chars().map(|c| c as u8).collect(),
        Some(other) => {
            return Err(Error::CorruptIndex(format!(
                "entry '{path}' has a prefix that is not a byte string: {other:?}"
            )))
        }
    };

    Ok(RawEntry {
        path,
        offset,
        length,
        prefix,
        segment_count,
    })
}

/// Read one index integer, refusing anything that is not a plain positive int.
fn decode_int(value: &Value, path: &str) -> Result<u64> {
    match value {
        Value::I64(n) if *n >= 0 => Ok(*n as u64),
        other => Err(Error::CorruptIndex(format!(
            "entry '{path}' has an offset or length that is not a positive integer: {other:?}"
        ))),
    }
}

/// Pickle and deflate an index, applying `key` to offsets and lengths.
///
/// Protocol 2 is what Ren'Py itself writes, and it is the only protocol both
/// Python 2 and Python 3 read, so an archive built here stays loadable by any
/// engine version that can load the format at all.
pub fn encode(entries: &[RawEntry], key: u64) -> Result<Vec<u8>> {
    let mut dict = BTreeMap::new();
    for entry in entries {
        dict.insert(
            HashableValue::String(entry.path.clone()),
            Value::List(vec![Value::Tuple(vec![
                Value::I64((entry.offset ^ key) as i64),
                Value::I64((entry.length ^ key) as i64),
                Value::Bytes(entry.prefix.clone()),
            ])]),
        );
    }

    let mut pickled = Vec::new();
    serde_pickle::value_to_writer(
        &mut pickled,
        &Value::Dict(dict),
        SerOptions::new().proto_v2(),
    )
    .map_err(|e| Error::CorruptIndex(format!("cannot pickle the index: {e}")))?;

    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(&mut encoder, &pickled)
        .and_then(|()| encoder.finish())
        .map_err(|e| Error::CorruptIndex(format!("cannot deflate the index: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, offset: u64, length: u64, prefix: &[u8]) -> RawEntry {
        RawEntry {
            path: path.to_string(),
            offset,
            length,
            prefix: prefix.to_vec(),
            segment_count: 1,
        }
    }

    #[test]
    fn encode_decode_round_trip_without_key() {
        let entries = vec![entry("a.rpy", 34, 10, b""), entry("b/c.png", 44, 7, b"xy")];
        let encoded = encode(&entries, 0).unwrap();
        let mut decoded = decode(&encoded[..], 0).unwrap();
        decoded.sort_by(|a, b| a.path.cmp(&b.path));
        assert_eq!(decoded, entries);
    }

    #[test]
    fn key_is_applied_to_offset_and_length_but_not_to_prefix() {
        let entries = vec![entry("a.rpy", 34, 10, b"pre")];
        let key = 0xdead_beef;
        let encoded = encode(&entries, key).unwrap();

        assert_eq!(decode(&encoded[..], key).unwrap(), entries);
        // Read with the wrong key, the prefix survives and the numbers do not.
        let wrong = decode(&encoded[..], 0).unwrap();
        assert_eq!(wrong[0].prefix, b"pre");
        assert_ne!(wrong[0].offset, 34);
    }

    #[test]
    fn refuses_a_stream_that_is_not_an_index() {
        assert!(decode(&b"not zlib at all"[..], 0).is_err());

        let pickled_list = {
            let mut out = Vec::new();
            serde_pickle::value_to_writer(&mut out, &Value::List(vec![]), SerOptions::new())
                .unwrap();
            let mut encoder =
                flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            std::io::Write::write_all(&mut encoder, &out).unwrap();
            encoder.finish().unwrap()
        };
        assert!(decode(&pickled_list[..], 0).is_err());
    }
}
