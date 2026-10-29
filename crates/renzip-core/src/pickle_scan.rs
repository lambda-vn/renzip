//! A pass over the opcodes of an index pickle, before serde-pickle decodes it.
//!
//! serde-pickle builds the whole value in memory and walks it recursively,
//! and two things a forged index can ask of it are not bounded by the size
//! of the stream:
//!
//! - a memoized container fetched again is copied out again, so one list of
//!   n items fetched n times is n² values out of a few kilobytes;
//! - containers nested a million deep overflow the stack while the value is
//!   converted or dropped, which aborts the process instead of failing.
//!
//! This pass runs the pickle machine without building anything: each stack
//! slot only records how deeply it nests. A real index never shares a
//! container (every list and tuple Ren'Py pickles is a fresh object, only
//! strings, bytes and globals come back from the memo) and nests four levels
//! at most, so refusing both costs no legitimate archive anything. With no
//! container shared, every value needs a byte of the stream of its own, and
//! what serde-pickle builds stays proportional to what it read.

use std::collections::HashMap;

use crate::error::IndexProblem;

/// Deepest nesting accepted. An index uses four levels at most: the dict, an
/// entry's list, a tuple, and the argument tuple of a bytes object pickled at
/// protocol 2.
pub const MAX_DEPTH: u32 = 16;

/// Nesting of a scalar: nothing is nested in it.
const SCALAR: u32 = 0;

/// Nesting of an empty container.
const EMPTY: u32 = 1;

/// Check that decoding `stream` stays bounded, up to its STOP opcode.
///
/// # Errors
///
/// [`IndexProblem::TooDeep`] and [`IndexProblem::SharedContainer`] for the
/// two shapes above, [`IndexProblem::Unpickle`] for a stream that is not a
/// pickle this crate can read at all.
pub fn check(stream: &[u8]) -> Result<(), IndexProblem> {
    let mut reader = Reader { stream, pos: 0 };
    let mut machine = Machine::default();
    loop {
        let opcode = reader.byte()?;
        if opcode == b'.' {
            return Ok(());
        }
        machine.run(opcode, &mut reader)?;
    }
}

/// A slot of the pickle stack.
enum Slot {
    /// A MARK opcode, delimiting the items of a container being built.
    Mark,
    /// A value, by how deeply it nests: 0 for a scalar, 1 for an empty
    /// container, one more than its deepest item otherwise.
    Value(u32),
}

/// The pickle machine, reduced to the nesting of what it holds.
#[derive(Default)]
struct Machine {
    stack: Vec<Slot>,
    /// Nesting of each memoized value, by memo id.
    memo: HashMap<u64, u32>,
}

impl Machine {
    /// Apply one opcode, reading its argument from `reader`.
    ///
    /// The opcodes are those of protocols 0 to 5. Persistent ids, extension
    /// codes, out-of-band buffers and class instances are refused: no index
    /// uses them, and serde-pickle cannot decode them either.
    fn run(&mut self, opcode: u8, reader: &mut Reader<'_>) -> Result<(), IndexProblem> {
        match opcode {
            b'(' => self.stack.push(Slot::Mark),
            b'0' => {
                self.stack.pop().ok_or_else(underflow)?;
            }
            b'1' => {
                self.pop_to_mark()?;
            }
            b'2' => self.dup()?,
            // Scalars, by the size of their argument.
            b'N' | 0x88 | 0x89 => self.push(SCALAR)?,
            b'K' => self.scalar(reader.take(1))?,
            b'M' => self.scalar(reader.take(2))?,
            b'J' => self.scalar(reader.take(4))?,
            b'G' => self.scalar(reader.take(8))?,
            b'F' | b'I' | b'L' | b'S' | b'V' => self.scalar(reader.line())?,
            b'U' | b'C' | 0x8a | 0x8c => self.scalar(reader.counted(1))?,
            b'T' | b'X' | b'B' | 0x8b => self.scalar(reader.counted(4))?,
            0x8d | 0x8e | 0x96 => self.scalar(reader.counted(8))?,
            b'c' => self.scalar(reader.line().and_then(|_| reader.line()))?,
            // Containers: new, built from the items above a mark or the
            // last n items, or grown by items added to the one below them.
            b']' | b'}' | b')' | 0x8f => self.push(EMPTY)?,
            b'l' | b't' | b'd' | 0x91 => {
                let items = self.pop_to_mark()?;
                self.push(items.max(EMPTY))?;
            }
            0x85..=0x87 => {
                let items = self.pop_items(opcode - 0x84)?;
                self.push(items.max(EMPTY))?;
            }
            b'a' => self.add_items(1)?,
            b's' => self.add_items(2)?,
            b'e' | b'u' | 0x90 => {
                let items = self.pop_to_mark()?;
                let container = self.pop()?;
                self.push(container.max(items))?;
            }
            // Calls, whose result serde-pickle turns into bytes or None.
            b'R' | 0x81 | 0x93 => self.call(2)?,
            0x92 => self.call(3)?,
            b'b' => {
                self.pop()?;
            }
            // Memo.
            b'p' => self.put(reader.decimal()?)?,
            b'q' => self.put(reader.uint(1)?)?,
            b'r' => self.put(reader.uint(4)?)?,
            0x94 => self.put(self.memo.len() as u64)?,
            b'g' => self.get(reader.decimal()?)?,
            b'h' => self.get(reader.uint(1)?)?,
            b'j' => self.get(reader.uint(4)?)?,
            // Protocol number and frame length: nothing to track.
            0x80 => {
                reader.take(1)?;
            }
            0x95 => {
                reader.take(8)?;
            }
            other => {
                return Err(malformed(&format!("unsupported opcode 0x{other:02x}")));
            }
        }
        Ok(())
    }

    /// Push a value, unless it nests too deeply.
    fn push(&mut self, depth: u32) -> Result<(), IndexProblem> {
        if depth > MAX_DEPTH {
            return Err(IndexProblem::TooDeep { limit: MAX_DEPTH });
        }
        self.stack.push(Slot::Value(depth));
        Ok(())
    }

    /// Push a scalar once its argument has been read.
    fn scalar<T>(&mut self, argument: Result<T, IndexProblem>) -> Result<(), IndexProblem> {
        argument?;
        self.push(SCALAR)
    }

    /// Pop a value, refusing a mark or an empty stack.
    fn pop(&mut self) -> Result<u32, IndexProblem> {
        match self.stack.pop() {
            Some(Slot::Value(depth)) => Ok(depth),
            _ => Err(underflow()),
        }
    }

    /// Pop `count` values, returning the nesting they give a container.
    fn pop_items(&mut self, count: u8) -> Result<u32, IndexProblem> {
        let mut nesting = SCALAR;
        for _ in 0..count {
            nesting = nesting.max(self.pop()?.saturating_add(1));
        }
        Ok(nesting)
    }

    /// Pop the values above the last mark and the mark itself, returning
    /// the nesting they give a container.
    fn pop_to_mark(&mut self) -> Result<u32, IndexProblem> {
        let mut nesting = SCALAR;
        loop {
            match self.stack.pop() {
                Some(Slot::Mark) => return Ok(nesting),
                Some(Slot::Value(depth)) => nesting = nesting.max(depth.saturating_add(1)),
                None => return Err(underflow()),
            }
        }
    }

    /// Add the last `count` values to the container just below them.
    fn add_items(&mut self, count: u8) -> Result<(), IndexProblem> {
        let items = self.pop_items(count)?;
        let container = self.pop()?;
        self.push(container.max(items))
    }

    /// Replace a callable and its `arguments - 1` arguments by its result.
    fn call(&mut self, arguments: u8) -> Result<(), IndexProblem> {
        self.pop_items(arguments)?;
        self.push(SCALAR)
    }

    /// Duplicate the top of the stack, which shares it if it is a container.
    fn dup(&mut self) -> Result<(), IndexProblem> {
        let depth = self.pop()?;
        if depth > SCALAR {
            return Err(IndexProblem::SharedContainer);
        }
        self.push(depth)?;
        self.push(depth)
    }

    /// Memoize the top of the stack under `id`.
    fn put(&mut self, id: u64) -> Result<(), IndexProblem> {
        let Some(Slot::Value(depth)) = self.stack.last() else {
            return Err(underflow());
        };
        self.memo.insert(id, *depth);
        Ok(())
    }

    /// Push the value memoized under `id`, unless it is a container.
    fn get(&mut self, id: u64) -> Result<(), IndexProblem> {
        match self.memo.get(&id) {
            None => Err(malformed(&format!(
                "memo id {id} is fetched before it is set"
            ))),
            Some(&depth) if depth > SCALAR => Err(IndexProblem::SharedContainer),
            Some(_) => self.push(SCALAR),
        }
    }
}

/// Reads opcode arguments off the stream.
struct Reader<'a> {
    stream: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// The next `count` bytes.
    fn take(&mut self, count: usize) -> Result<&'a [u8], IndexProblem> {
        let end = self.pos.checked_add(count).ok_or_else(ends_early)?;
        let bytes = self.stream.get(self.pos..end).ok_or_else(ends_early)?;
        self.pos = end;
        Ok(bytes)
    }

    /// The next byte.
    fn byte(&mut self) -> Result<u8, IndexProblem> {
        self.take(1)?.first().copied().ok_or_else(ends_early)
    }

    /// A little-endian unsigned integer of `width` bytes, at most 8.
    fn uint(&mut self, width: usize) -> Result<u64, IndexProblem> {
        let bytes = self.take(width)?;
        Ok(bytes
            .iter()
            .rev()
            .fold(0, |value, &byte| (value << 8) | u64::from(byte)))
    }

    /// Skip a byte string whose length precedes it on `width` bytes.
    fn counted(&mut self, width: usize) -> Result<(), IndexProblem> {
        let length = usize::try_from(self.uint(width)?).map_err(|_| ends_early())?;
        self.take(length).map(drop)
    }

    /// The bytes up to the next newline, which is consumed too.
    fn line(&mut self) -> Result<&'a [u8], IndexProblem> {
        let rest = self.stream.get(self.pos..).ok_or_else(ends_early)?;
        let length = rest
            .iter()
            .position(|&byte| byte == b'\n')
            .ok_or_else(ends_early)?;
        let line = self.take(length)?;
        self.take(1)?;
        Ok(line)
    }

    /// A memo id written as decimal digits on a line of its own.
    fn decimal(&mut self) -> Result<u64, IndexProblem> {
        std::str::from_utf8(self.line()?)
            .ok()
            .and_then(|digits| digits.trim_end_matches('\r').parse().ok())
            .ok_or_else(|| malformed("memo id is not a decimal number"))
    }
}

/// A stream this pass cannot make sense of.
fn malformed(what: &str) -> IndexProblem {
    IndexProblem::Unpickle {
        error: what.to_string(),
    }
}

/// An opcode popping more than the stack holds.
fn underflow() -> IndexProblem {
    malformed("an opcode takes more values than the stack holds")
}

/// A stream ending in the middle of an opcode or its argument.
fn ends_early() -> IndexProblem {
    malformed("the stream ends inside an opcode")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_list_fetched_from_the_memo_is_refused() {
        // [x] * 3 where x = [None, None]: x is copied out once per fetch.
        let stream = b"\x80\x02(NNlq\x00(h\x00h\x00h\x00l.";
        assert!(matches!(check(stream), Err(IndexProblem::SharedContainer)));
    }

    #[test]
    fn a_list_filled_after_it_is_memoized_is_still_a_container() {
        // Memoized while empty, then filled: fetching it is the same copy.
        let stream = b"\x80\x02]q\x00(NNe(h\x00h\x00l.";
        assert!(matches!(check(stream), Err(IndexProblem::SharedContainer)));
    }

    #[test]
    fn a_scalar_fetched_from_the_memo_is_accepted() {
        // {"a": [(1, 2, b"")], "b": [(3, 4, b"")]} with b"" fetched back.
        let stream = b"\x80\x02}(X\x01\x00\x00\x00a]K\x01K\x02C\x00q\x00\x87a\
                       X\x01\x00\x00\x00b]K\x03K\x04h\x00\x87au.";
        assert!(check(stream).is_ok());
    }

    #[test]
    fn nesting_past_the_limit_is_refused() {
        let depth = MAX_DEPTH as usize + 1;
        let mut stream = vec![0x80, 2];
        stream.extend(std::iter::repeat_n(b']', depth));
        stream.extend(std::iter::repeat_n(b'a', depth - 1));
        stream.push(b'.');
        assert!(matches!(
            check(&stream),
            Err(IndexProblem::TooDeep { limit: MAX_DEPTH })
        ));
    }

    #[test]
    fn a_stream_cut_inside_an_opcode_is_refused() {
        assert!(matches!(
            check(b"\x80\x02X\xff\x00\x00\x00ab"),
            Err(IndexProblem::Unpickle { .. })
        ));
    }
}
