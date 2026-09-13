//! Decoding and encoding of the archive index: zlib over a Python pickle.
//!
//! The index is the only compressed part of an .rpa; file data is stored raw.
//! Cross-checked against renpy/loader.py: the engine reads from the header
//! offset to the end of the file, inflates it, and unpickles a dict mapping a
//! name to a list of tuples. A single tuple holds (offset, length) or
//! (offset, length, prefix); for RPA-3.x, offset and length are XORed with
//! the header key while prefix is not. A list of several tuples is read
//! differently: the engine concatenates the bytes each `(offset, length)`
//! pair names, in order, and never reads a prefix from one - unpacking a
//! three-field tuple that way raises `ValueError`, which is why this crate
//! refuses such an entry rather than guessing which field to drop.
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
    /// Byte ranges inside the archive holding this entry's data, as
    /// `(offset, length)` pairs with the header key already applied, in the
    /// order Ren'Py itself reads and concatenates them.
    ///
    /// Empty when the index listed more than one tuple and at least one of
    /// them was not a plain `(offset, length)` pair: Ren'Py's own loader
    /// raises `ValueError` trying to read such an entry rather than reading
    /// part of it, so this crate leaves it unreadable too instead of
    /// guessing which field was meant.
    pub segments: Vec<(u64, u64)>,
    /// Bytes to prepend to the data read at the entry's one segment.
    ///
    /// Only ever set when there is exactly one segment: Ren'Py never reads a
    /// prefix from a multi-segment entry.
    pub prefix: Vec<u8>,
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
    if items.is_empty() {
        // Ren'Py itself indexes the empty list to read the first tuple and
        // raises IndexError; there is no partial read to fall back to, so
        // this is fatal the same way a stream that does not unpickle at all
        // is, rather than something to route around one entry at a time.
        return Err(Error::CorruptIndex(format!("entry '{path}' has no tuple")));
    }

    if items.len() == 1 {
        let fields = as_tuple_fields(&path, items.into_iter().next().unwrap())?;
        if fields.len() != 2 && fields.len() != 3 {
            return Err(Error::CorruptIndex(format!(
                "entry '{path}' has {} fields, expected 2 or 3",
                fields.len()
            )));
        }
        let offset = decode_int(&fields[0], &path)? ^ key;
        let length = decode_int(&fields[1], &path)? ^ key;
        let prefix = decode_prefix(&path, fields.get(2))?;
        return Ok(RawEntry {
            path,
            segments: vec![(offset, length)],
            prefix,
        });
    }

    // More than one tuple: Ren'Py concatenates the (offset, length) pair
    // each one names. It never expects a third field there, and unpacking
    // one that has it raises ValueError - an entry like that is left
    // unreadable (empty segments) rather than guessed at, and the caller
    // reports it as rejected instead of failing the whole archive over it.
    let mut segments = Vec::with_capacity(items.len());
    for item in items {
        let fields = as_tuple_fields(&path, item)?;
        if fields.len() != 2 {
            return Ok(RawEntry {
                path,
                segments: Vec::new(),
                prefix: Vec::new(),
            });
        }
        let offset = decode_int(&fields[0], &path)? ^ key;
        let length = decode_int(&fields[1], &path)? ^ key;
        segments.push((offset, length));
    }
    Ok(RawEntry {
        path,
        segments,
        prefix: Vec::new(),
    })
}

/// Unwrap a pickled tuple's fields, or fail: a value that is not even a
/// tuple is corrupt data, not an unusual but readable shape.
fn as_tuple_fields(path: &str, value: Value) -> Result<Vec<Value>> {
    let Value::Tuple(fields) = value else {
        return Err(Error::CorruptIndex(format!(
            "entry '{path}' is not described by a tuple"
        )));
    };
    Ok(fields)
}

/// Decode a tuple's optional third field into prefix bytes.
fn decode_prefix(path: &str, field: Option<&Value>) -> Result<Vec<u8>> {
    match field {
        None => Ok(Vec::new()),
        Some(Value::Bytes(b)) => Ok(b.clone()),
        // An empty bytes object written at protocol 2, once its unmodelled
        // constructor has been replaced by None. Ren'Py stores exactly that
        // for every entry it packs without a prefix.
        Some(Value::None) => Ok(Vec::new()),
        // A prefix pickled by Python 2 arrives as a byte string too, but one
        // pickled from a unicode object arrives as a String; Ren'Py writes it
        // back out as latin-1, so that is how it is turned into bytes here.
        Some(Value::String(s)) => Ok(s.chars().map(|c| c as u8).collect()),
        Some(other) => Err(Error::CorruptIndex(format!(
            "entry '{path}' has a prefix that is not a byte string: {other:?}"
        ))),
    }
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
/// Protocol 2 is a deliberate choice, not what Ren'Py itself writes: its own
/// archiver pickles at `HIGHEST_PROTOCOL`, which is 2 on the Python 2
/// releases (up to 7.x) and 5 on Ren'Py 8's Python 3. Protocol 2 is the
/// newest one both still read, so an archive built here stays loadable by
/// either.
///
/// Every `entry` is written as a single tuple: this crate never builds a
/// multi-segment archive, so `entry.segments` always holds exactly one
/// range here.
pub fn encode(entries: &[RawEntry], key: u64) -> Result<Vec<u8>> {
    let mut dict = BTreeMap::new();
    for entry in entries {
        let (offset, length) = entry.segments[0];
        dict.insert(
            HashableValue::String(entry.path.clone()),
            Value::List(vec![Value::Tuple(vec![
                Value::I64((offset ^ key) as i64),
                Value::I64((length ^ key) as i64),
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
            segments: vec![(offset, length)],
            prefix: prefix.to_vec(),
        }
    }

    /// Pickle (protocol 2) and deflate a [`Value`], the way an index is
    /// stored, for tests that need to hand-build one no [`encode`] call
    /// would produce.
    fn compress_pickle(value: &Value) -> Vec<u8> {
        let mut out = Vec::new();
        serde_pickle::value_to_writer(&mut out, value, SerOptions::new()).unwrap();
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        std::io::Write::write_all(&mut encoder, &out).unwrap();
        encoder.finish().unwrap()
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
        assert_ne!(wrong[0].segments[0].0, 34);
    }

    #[test]
    fn multiple_two_field_tuples_are_kept_as_segments_to_concatenate() {
        let mut dict = BTreeMap::new();
        dict.insert(
            HashableValue::String("twice.txt".into()),
            Value::List(vec![
                Value::Tuple(vec![Value::I64(0), Value::I64(5)]),
                Value::Tuple(vec![Value::I64(5), Value::I64(6)]),
            ]),
        );
        let pickled = compress_pickle(&Value::Dict(dict));

        assert_eq!(
            decode(&pickled[..], 0).unwrap(),
            vec![entry_with_segments("twice.txt", &[(0, 5), (5, 6)], b"")]
        );
    }

    #[test]
    fn a_three_field_tuple_among_several_is_left_with_no_segment() {
        // Ren'Py's own loader unpacks every tuple of a multi-tuple entry as
        // an (offset, length) pair; one with a third field raises
        // ValueError there instead of reading part of the entry.
        let mut dict = BTreeMap::new();
        dict.insert(
            HashableValue::String("bad.txt".into()),
            Value::List(vec![
                Value::Tuple(vec![Value::I64(0), Value::I64(5), Value::Bytes(vec![])]),
                Value::Tuple(vec![Value::I64(5), Value::I64(6), Value::Bytes(vec![])]),
            ]),
        );
        let pickled = compress_pickle(&Value::Dict(dict));

        let decoded = decode(&pickled[..], 0).unwrap();
        assert_eq!(decoded, vec![entry_with_segments("bad.txt", &[], b"")]);
    }

    fn entry_with_segments(path: &str, segments: &[(u64, u64)], prefix: &[u8]) -> RawEntry {
        RawEntry {
            path: path.to_string(),
            segments: segments.to_vec(),
            prefix: prefix.to_vec(),
        }
    }

    #[test]
    fn refuses_a_stream_that_is_not_an_index() {
        assert!(decode(&b"not zlib at all"[..], 0).is_err());
        assert!(decode(&compress_pickle(&Value::List(vec![]))[..], 0).is_err());
    }
}
