//! A zone's script: the compiled Pawn in section 2 of its zone file, read only as far as what a
//! sign shows (spike 107).
//!
//! ```text
//! header  +0x00 u32 size, +0x04 u16 magic 0xf1e0, +0x08 u16 flags (0x04 the code compact),
//!         +0x0c u32 where the code starts, +0x10 u32 where the data starts, +0x1c u32 the
//!         entry point, in bytes from the code's start
//! code    32-bit cells, compact-encoded as groups of seven bits, the most significant first, a
//!         set top bit where another group follows and bit 6 of a cell's first group its sign
//! ```
//!
//! The entry point switches on the number of what the player spoke to or read, a furniture
//! record's +0x00 among them. A sign's case calls a function of its own, which shows the sign's
//! message by one call with two constants: the message's line in the zone's text file, and its
//! style, 5 a town's or a street's sign, 6 a building's plate, 7 and 8 tips, 9 a route's name. A
//! number from 2000 up is one of the game's shared scripts, not the zone's.
//!
//! Jumps and calls give their target in bytes from the instruction; opcodes from 160 up are packed,
//! their one operand in the cell's top half.

use std::collections::HashSet;
use std::sync::OnceLock;

use indexmap::IndexMap;

use crate::core::binary;
use crate::py::{self, Error, Result};

pub const MAGIC: i64 = 0xF1E0;

/// `struct.Struct("<IHBBHHIIIII")`: the header's eleven fields, little-endian, unpadded.
pub struct Header;

pub const HEADER: Header = Header;

impl Header {
    /// `HEADER.size`.
    pub fn size(&self) -> i64 {
        32
    }

    /// `HEADER.unpack_from(blob)`.
    pub fn unpack_from(&self, blob: &[u8]) -> Result<[i64; 11]> {
        if (blob.len() as i64) < self.size() {
            return Err(Error::unpack(format!(
                "unpack_from requires a buffer of at least {} bytes for unpacking {} bytes at \
                 offset 0 (actual buffer size is {})",
                self.size(),
                self.size(),
                blob.len()
            )));
        }
        Ok([
            binary::u32(blob, 0)?,
            binary::u16(blob, 4)?,
            i64::from(blob[6]),
            i64::from(blob[7]),
            binary::u16(blob, 8)?,
            binary::u16(blob, 10)?,
            binary::u32(blob, 12)?,
            binary::u32(blob, 16)?,
            binary::u32(blob, 20)?,
            binary::u32(blob, 24)?,
            binary::u32(blob, 28)?,
        ])
    }

    /// `HEADER.pack(*values)`: a value out of its field's range is `struct.error`.
    pub fn pack(&self, values: [i64; 11]) -> Result<Vec<u8>> {
        const FIELDS: [(char, usize, i64); 11] = [
            ('I', 4, 0xFFFF_FFFF),
            ('H', 2, 0xFFFF),
            ('B', 1, 0xFF),
            ('B', 1, 0xFF),
            ('H', 2, 0xFFFF),
            ('H', 2, 0xFFFF),
            ('I', 4, 0xFFFF_FFFF),
            ('I', 4, 0xFFFF_FFFF),
            ('I', 4, 0xFFFF_FFFF),
            ('I', 4, 0xFFFF_FFFF),
            ('I', 4, 0xFFFF_FFFF),
        ];
        let mut out = Vec::new();
        for (value, (code, width, max)) in values.into_iter().zip(FIELDS) {
            if !(0..=max).contains(&value) {
                return Err(Error::unpack(format!(
                    "'{code}' format requires 0 <= number <= {max}"
                )));
            }
            out.extend_from_slice(&value.to_le_bytes()[..width]);
        }
        Ok(out)
    }
}

pub const COMPACT: i64 = 0x04;
pub const RET: i64 = 47;
pub const RETN: i64 = 48;
pub const CALL: i64 = 49;
pub const JUMP: i64 = 51;
pub const SWITCH: i64 = 129;
pub const CASETBL: i64 = 130;
/// Packed: push the constant in the top half. A call's arguments are pushed so, the last first,
/// and then their size in bytes.
pub const PUSH_CONSTANT: i64 = 0xBC;
/// A function's first instruction, and a call to one of the game's natives: the native's number
/// and its arguments' size in bytes.
pub const PROC: i64 = 46;
pub const NATIVE: i64 = 135;
/// The first number of the game's shared scripts (`Script.runs`).
pub const SHARED: i64 = 2000;
pub const PACKED: i64 = 160;
/// The style a building's plate is shown in (`sign_lines`).
pub const PLATE: i64 = 6;

fn one() -> &'static HashSet<i64> {
    static ONE: OnceLock<HashSet<i64>> = OnceLock::new();
    ONE.get_or_init(|| {
        (1..9)
            .chain(10..23)
            .chain([24, 26, 28, 29, 30, 31, 32, 38, 39, 40, 41, 44, 45, 49])
            .chain(51..65)
            .chain([68, 69, 70, 71, 87, 88, 91, 92, 105, 106, 109, 110, 114, 115])
            .chain(117..122)
            .chain([123, 129, 133, 136, 158])
            .collect()
    })
}

fn two() -> &'static HashSet<i64> {
    static TWO: OnceLock<HashSet<i64>> = OnceLock::new();
    TWO.get_or_init(|| HashSet::from([125, 127, 135, 154, 155, 156, 157, 159]))
}

/// Pawn's push of two to five values at once, each value a cell.
fn many() -> &'static IndexMap<i64, i64> {
    static MANY: OnceLock<IndexMap<i64, i64>> = OnceLock::new();
    MANY.get_or_init(|| {
        (138..154)
            .map(|op| (op, 2 + py::floordiv(op - 138, 4)))
            .collect()
    })
}

/// How far into the entry point its switch is looked for, and a case's first call.
pub const SEARCH: i64 = 20;

/// A zone script's code, each instruction by the cell it starts at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    pub entry: i64,
    pub instructions: IndexMap<i64, (i64, Vec<i64>)>,
    pub order: Vec<i64>,
}

// `tuple.index(x)` raises where `x` is absent; a `Script` from `read` always has it.
#[track_caller]
fn index_of(order: &[i64], start: i64) -> Result<usize> {
    match order.iter().position(|&cell| cell == start) {
        Some(at) => Ok(at),
        None => Err(Error::value("tuple.index(x): x not in tuple")),
    }
}

impl Script {
    /// The entry point's cases: the number it is run for -> the cell its case starts at.
    pub fn cases(&self) -> Result<IndexMap<i64, i64>> {
        let Some(found) = self.first(self.entry, SWITCH)? else {
            return Ok(IndexMap::new());
        };
        let table = self.target(found)?;
        let (op, operands) = match self.instructions.get(&table) {
            Some((op, operands)) => (Some(*op), operands.as_slice()),
            None => (None, &[][..]),
        };
        if op != Some(CASETBL) || operands.is_empty() {
            return Ok(IndexMap::new());
        }
        let count = operands[0];
        let mut cases = IndexMap::new();
        for j in 0..count {
            if 3 + 2 * j < operands.len() as i64 {
                cases.insert(
                    operands[(2 + 2 * j) as usize],
                    table + 3 + 2 * j + py::floordiv(operands[(3 + 2 * j) as usize], 4),
                );
            }
        }
        Ok(cases)
    }

    /// The function a case at `start` calls before it jumps anywhere, or None.
    pub fn called(&self, start: i64) -> Result<Option<i64>> {
        let at = if self.instructions.contains_key(&start) {
            index_of(&self.order, start)? as i64
        } else {
            self.order.len() as i64
        };
        for &cell in py::slice(&self.order, at, at + SEARCH / 3) {
            let (op, _) = &self.instructions[&cell];
            if *op == CALL {
                return Ok(Some(self.target(cell)?));
            }
            if *op == JUMP {
                return Ok(None);
            }
        }
        Ok(None)
    }

    /// The constants each call in the function at `start` is made with, for every call whose
    /// arguments are all pushed as constants, the first argument first.
    ///
    /// A Python generator there; every call is found at once here, which only a reader that stops
    /// early could tell apart.
    pub fn constant_calls(&self, start: i64) -> Result<Vec<Vec<i64>>> {
        let body = self.body(start)?;
        let mut calls = Vec::new();
        for (n, &cell) in body.iter().enumerate() {
            if self.instructions[&cell].0 != CALL || n == 0 {
                continue;
            }
            let (op, size) = &self.instructions[&body[n - 1]];
            if *op != PUSH_CONSTANT {
                continue;
            }
            let Some(size) = size.first() else {
                return Err(Error::index("tuple index out of range"));
            };
            let count = py::floordiv(*size, 4);
            let pushed: Vec<&(i64, Vec<i64>)> = (0..count)
                .filter(|m| n as i64 - 2 - m >= 0)
                .map(|m| &self.instructions[&body[(n as i64 - 2 - m) as usize]])
                .collect();
            if pushed.len() as i64 == count && pushed.iter().all(|(op, _)| *op == PUSH_CONSTANT) {
                let mut constants = Vec::new();
                for (_, operands) in pushed {
                    let Some(constant) = operands.first() else {
                        return Err(Error::index("tuple index out of range"));
                    };
                    constants.push(*constant);
                }
                calls.push(constants);
            }
        }
        Ok(calls)
    }

    /// The shared script the case `number` runs, or None: its first call is to a function
    /// that pushes one constant from `SHARED` up and hands it to a native, as every vending
    /// machine's case does, 0x28f0 to 0x28f4 by where it stands (spike 210).
    pub fn runs(&self, number: i64) -> Result<Option<i64>> {
        let found = self.native(number)?;
        Ok(found
            .filter(|&(_, constant)| constant >= SHARED)
            .map(|(_, constant)| constant))
    }

    /// The native the case `number` hands one constant to, and the constant, or None
    /// (`call`): as a vending machine's does (`runs`) and each of Coumarine Gym's vines, 56 to
    /// swing on one and 57 to climb one, with the end's number among its kind (spike 250).
    pub fn native(&self, number: i64) -> Result<Option<(i64, i64)>> {
        Ok(match self.call(number)? {
            Some((native, constants)) if constants.len() == 1 => Some((native, constants[0])),
            _ => None,
        })
    }

    /// The native the case `number` hands constants to, and the constants, the first
    /// argument first, or None: its first call is to a function that pushes only constants,
    /// the last argument first, and calls the native with their size. Each of Laverre Gym's
    /// panels hands native 56 three, x, the floor in squares of height and z (spike 258).
    pub fn call(&self, number: i64) -> Result<Option<(i64, Vec<i64>)>> {
        let case = self.cases()?.get(&number).copied();
        let function = match case {
            None => None,
            Some(case) => self.called(case)?,
        };
        let ops: Vec<&(i64, Vec<i64>)> = match function {
            None => Vec::new(),
            Some(function) => self
                .body(function)?
                .iter()
                .map(|cell| &self.instructions[cell])
                .collect(),
        };
        if ops.is_empty() || ops[0].0 != PROC {
            return Ok(None);
        }
        let pushed: Vec<&(i64, Vec<i64>)> = ops[1..]
            .iter()
            .copied()
            .take_while(|one| one.0 == PUSH_CONSTANT)
            .collect();
        let native = ops.get(1 + pushed.len()).copied();
        let Some(native) = native.filter(|native| {
            native.0 == NATIVE && py::slice_from(&native.1, 1) == [4 * pushed.len() as i64]
        }) else {
            return Ok(None);
        };
        let Some(&called) = native.1.first() else {
            return Err(Error::index("tuple index out of range"));
        };
        let mut constants: Vec<i64> = Vec::new();
        for (_, operands) in pushed.iter().rev() {
            let Some(&constant) = operands.first() else {
                return Err(Error::index("tuple index out of range"));
            };
            constants.push(constant);
        }
        Ok(Some((called, constants)))
    }

    /// The cells of the function at `start`, up to its first return.
    fn body(&self, start: i64) -> Result<Vec<i64>> {
        if !self.instructions.contains_key(&start) {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        let at = index_of(&self.order, start)?;
        for &cell in &self.order[at..] {
            out.push(cell);
            let op = self.instructions[&cell].0;
            if op == RET || op == RETN {
                break;
            }
        }
        Ok(out)
    }

    fn first(&self, start: i64, wanted: i64) -> Result<Option<i64>> {
        if !self.instructions.contains_key(&start) {
            return Ok(None);
        }
        let at = index_of(&self.order, start)? as i64;
        Ok(py::slice(&self.order, at, at + SEARCH)
            .iter()
            .copied()
            .find(|cell| self.instructions[cell].0 == wanted))
    }

    fn target(&self, cell: i64) -> Result<i64> {
        let Some((_, operands)) = self.instructions.get(&cell) else {
            return Err(Error::key(cell.to_string()));
        };
        let Some(first) = operands.first() else {
            return Err(Error::index("tuple index out of range"));
        };
        Ok(cell + py::floordiv(*first, 4))
    }
}

/// A zone's script from its zone file's section 2, or None for one that is not a script.
///
/// Raises `IndexError` for a script whose case table's count walks the decoder backwards out of
/// the code, where Python's list index raised it (a negative index first wraps round the list).
pub fn read(blob: &[u8]) -> Result<Option<Script>> {
    if (blob.len() as i64) < HEADER.size() {
        return Ok(None);
    }
    let [
        size,
        magic,
        _file,
        _version,
        flags,
        _entry_size,
        code,
        data,
        _heap,
        _stack,
        entry,
    ] = HEADER
        .unpack_from(blob)
        .expect("a blob of at least HEADER.size bytes unpacks");
    // The data's offset is into the code as it expands, so past the end of a compact one.
    if magic != MAGIC || !(code <= size && size <= blob.len() as i64) || data < code {
        return Ok(None);
    }
    let body = py::slice(blob, code, size);
    let cells = if flags & COMPACT != 0 {
        expand(body)
    } else {
        body.as_chunks::<4>()
            .0
            .iter()
            .map(|cell| i64::from(i32::from_le_bytes(*cell)))
            .collect()
    };
    let instructions: IndexMap<i64, (i64, Vec<i64>)> =
        decode(py::slice(&cells, 0, py::floordiv(data - code, 4)))?
            .into_iter()
            .collect();
    let order = instructions.keys().copied().collect();
    Ok(Some(Script {
        entry: py::floordiv(entry, 4),
        instructions,
        order,
    }))
}

/// (line, style) of each message the sign `number` shows by the two constants a sign's are
/// shown by, or none for a number the script has no case for.
pub fn sign_lines(script: &Script, number: i64) -> Result<Vec<(i64, i64)>> {
    let cases = script.cases()?;
    let case = cases.get(&number);
    let function = match case {
        None => None,
        Some(&case) => script.called(case)?,
    };
    let Some(function) = function else {
        return Ok(Vec::new());
    };
    Ok(script
        .constant_calls(function)?
        .iter()
        .filter(|pair| pair.len() == 2)
        .map(|pair| (pair[0], pair[1]))
        .collect())
}

fn expand(body: &[u8]) -> Vec<i64> {
    let mut cells = Vec::new();
    let (mut value, mut first) = (0_i64, true);
    for &byte in body {
        if first {
            (value, first) = (if byte & 0x40 != 0 { -1 } else { 0 }, false);
        }
        // Bits shifted out of the top are lost, where Python's int grows; only the low 32 bits
        // are kept below, and shifting left never changes those.
        value = value << 7 | i64::from(byte & 0x7F);
        if byte & 0x80 == 0 {
            value &= 0xFFFF_FFFF;
            cells.push(if value & 0x8000_0000 != 0 {
                value - (1 << 32)
            } else {
                value
            });
            (value, first) = (0, true);
        }
    }
    cells
}

/// `code[index]`: a negative index counts from the end, and one out of range is `IndexError`.
#[track_caller]
fn item(code: &[i64], index: i64) -> Result<i64> {
    let at = if index < 0 {
        index + code.len() as i64
    } else {
        index
    };
    match usize::try_from(at).ok().and_then(|at| code.get(at)) {
        Some(&cell) => Ok(cell),
        None => Err(Error::index("list index out of range")),
    }
}

/// (cell, (opcode, operands)) for every instruction, a case table's count, default and pairs
/// as its operands.
fn decode(code: &[i64]) -> Result<Vec<(i64, (i64, Vec<i64>))>> {
    let mut out = Vec::new();
    let mut cell = 0_i64;
    while cell < code.len() as i64 {
        let word = item(code, cell)? & 0xFFFF_FFFF;
        let op = word & 0xFFFF;
        if op == CASETBL && cell + 1 < code.len() as i64 {
            let count = item(code, cell + 1)?;
            out.push((
                cell,
                (op, py::slice(code, cell + 1, cell + 3 + 2 * count).to_vec()),
            ));
            cell += 3 + 2 * count;
            continue;
        }
        if op >= PACKED {
            let top = word >> 16;
            out.push((
                cell,
                (
                    op,
                    vec![if top & 0x8000 != 0 {
                        top - 0x10000
                    } else {
                        top
                    }],
                ),
            ));
            cell += 1;
            continue;
        }
        let count = if one().contains(&op) {
            1
        } else if two().contains(&op) {
            2
        } else {
            many().get(&op).copied().unwrap_or(0)
        };
        out.push((
            cell,
            (op, py::slice(code, cell + 1, cell + 1 + count).to_vec()),
        ));
        cell += 1 + count;
    }
    Ok(out)
}
