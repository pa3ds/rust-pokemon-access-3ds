//! Sun's zone scripts: which line of text a thing, a sign or a person shows (spike A).
//!
//! An area's eighth file in the event archive ('ZS', `zones.AREA_FILES`) is a pack of one script a
//! zone, in the area's order. Each is a Pawn program, written from Pawn's own documentation of its
//! format and nobody's code:
//!
//! ```text
//! +0x00 u32 the image's size in the file    +0x04 u16 0xf1e0    +0x08 u16 flags, 0x1c: compact
//! +0x0c u32 cod, the code's start    +0x10 u32 dat, the data's    +0x1c u32 cip, `main`, from
//! the code's start    +0x24 natives, (u32 0, u32 a hash of the name), up to +0x28's table
//! ```
//!
//! From `cod` to the file's end the image is in Pawn's compact encoding: a cell is 7 bits a byte, the
//! high bit set on every byte but its last, and the first byte's 0x40 its sign. The code is Pawn 3's
//! instructions, and those that take one small operand are packed: the opcode in the low half of
//! a cell and the operand in the high half (`PUSH_P_C`, 0xbc, pushes a constant).
//!
//! `main` switches on a number the game hands it, a **case**: a thing's +0x30 (`zones.THING_CODE`)
//! or a person's +0x38 (`zones.PERSON_TALK`), 1 to 15 on Route 1. Each case calls one function, and
//! that function shows its words by calling a helper with the line first, of the zone's own story
//! text file for a thing or a person and of story file 21 for a sign: `(5, 0, 1)` is the bedroom's
//! "It's a Wii U!", `(4)` the sign of Route 1's "Pokémon Research Lab". Every such helper comes to
//! the native `MESSAGE`, the message window, within a few calls, so the line is the first constant
//! passed to the first call in the case's function that does. A case whose words are chosen as the
//! story goes, or that shows none, has no line here.
//!
//! A person's case past 0xff names a script of the global archive (`a/0/9/2`), not the zone's, and
//! is not read.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;

use crate::core::binary::{u16, u32};
use crate::py::{self, Error, Kind, Result};

pub const MAGIC: i64 = 0xF1E0;
pub const COMPACT: i64 = 0x04;
pub const HEADER: i64 = 0x2C;
/// The hash a script names the message window's native by: every helper that shows a line of
/// text comes to it, in every zone measured (spike A).
pub const MESSAGE: i64 = 0xFD1CF919;

pub const PROC: i64 = 46;
pub const RET: i64 = 47;
pub const RETN: i64 = 48;
pub const CALL: i64 = 49;
pub const PUSH_C: i64 = 39;
pub const SWITCH: i64 = 129;
pub const CASETBL: i64 = 130;
pub const SYSREQ_N: i64 = 135;
/// Opcodes from here take their operand in the cell's high half.
pub const PACKED: i64 = 0xA0;
pub const PUSH_P_C: i64 = 0xBC;
/// Pawn's PUSH2_C to PUSH5_C, each pushing its operands in order.
pub const PUSH_N_C: [(i64, i64); 4] = [(138, 2), (142, 3), (146, 4), (150, 5)];
/// A `frozenset` only tested.
pub const NO_OPERAND: [i64; 60] = [
    0, 9, 23, 25, 27, 33, 34, 35, 36, 37, 42, 43, 46, 47, 48, 50, 65, 66, 67, 72, 73, 74, 75, 76,
    77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 89, 90, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103,
    104, 107, 108, 111, 112, 113, 116, 122, 128, 131, 132, 134, 137,
];
/// LINE, SRANGE, SYSREQ_N, LOAD_BOTH, LOAD_S_BOTH, CONST and CONST_S; the macro pushes take as
/// many as `PUSH_N_C` says, and their PUSHn, PUSHn_S and PUSHn_ADR as many again.
pub const TWO_OPERANDS: [i64; 7] = [125, 127, 135, 154, 155, 156, 157];
/// Calls followed down to find the message window: the bedroom's are three deep.
pub const DEPTH: i64 = 6;
/// A person's case from here is a global script's (`a/0/9/2`).
pub const TALK_CASES: i64 = 0x100;

/// (address, opcode, operands).
pub type Instruction = (i64, i64, Vec<i64>);

/// `PUSH_N_C[op]`, or None for an opcode not in it.
fn push_n_c(op: i64) -> Option<i64> {
    PUSH_N_C
        .iter()
        .find(|(code, _)| *code == op)
        .map(|(_, count)| *count)
}

/// `args[i]`, or the IndexError Python raises past its end.
#[track_caller]
fn arg(args: &[i64], i: usize) -> Result<i64> {
    args.get(i)
        .copied()
        .ok_or_else(|| Error::index("tuple index out of range"))
}

/// One zone's script: the case `main` runs for each thing and person, and the line each
/// shows first.
pub struct ZoneScript {
    natives: Vec<i64>,
    code: Vec<Instruction>,
    at: HashMap<i64, usize>,
    cases: IndexMap<i64, i64>,
    reaches: RefCell<HashMap<i64, bool>>,
}

impl ZoneScript {
    /// Python's `ScriptError` is a ValueError here, which `py::Kind` has no class of its own for:
    /// nothing else in reading a script raises one, so `scripts` catches it as Python's does.
    pub fn new(data: &[u8]) -> Result<ZoneScript> {
        if (data.len() as i64) < HEADER || u16(data, 4)? != MAGIC || u16(data, 8)? & COMPACT == 0 {
            return Err(Error::new(Kind::ValueError, "not a compact Pawn script"));
        }
        let (cod, dat, cip) = (u32(data, 0x0C)?, u32(data, 0x10)?, u32(data, 0x1C)?);
        let (natives, after) = (u32(data, 0x24)?, u32(data, 0x28)?);
        if !(HEADER <= natives && natives <= after && after <= cod && cod <= data.len() as i64)
            || dat < cod
        {
            return Err(Error::new(
                Kind::ValueError,
                "a script whose tables run past it",
            ));
        }
        let mut read: Vec<i64> = Vec::new();
        for at in (natives..after).step_by(8) {
            read.push(u32(data, at + 4)?);
        }
        let expanded = expanded(py::slice(data, 0, u32(data, 0)?), cod);
        let cells = py::slice(&expanded, 0, (dat - cod) / 4);
        let code = decoded(cells);
        let at = code
            .iter()
            .enumerate()
            .map(|(index, (address, _op, _args))| (*address, index))
            .collect();
        let mut script = ZoneScript {
            natives: read,
            code,
            at,
            cases: IndexMap::new(),
            reaches: RefCell::new(HashMap::new()),
        };
        script.cases = script.switch(cip)?;
        Ok(script)
    }

    /// A `frozenset` only compared.
    pub fn cases(&self) -> HashSet<i64> {
        self.cases.keys().copied().collect()
    }

    /// The line the case shows first, or None where it shows none read here. `count` keeps
    /// to a call of that many arguments: 1 for a sign's helper.
    pub fn line(&self, case: i64, count: Option<usize>) -> Result<Option<i64>> {
        let start = self.cases.get(&case).copied();
        let function = match start {
            None => None,
            Some(start) => self.first_call(start)?,
        };
        let Some(function) = function else {
            return Ok(None);
        };
        let mut pushed: Vec<Option<i64>> = Vec::new();
        for (address, op, args) in self.function(function) {
            if *op == CALL {
                let target = (address + signed(arg(args, 0)?)) & 0xFFFFFFFF;
                let arguments = self::arguments(&pushed);
                pushed = Vec::new();
                let Some(arguments) = arguments else {
                    continue;
                };
                if !self.shows(target, DEPTH)? {
                    continue;
                }
                if count.is_none_or(|count| arguments.len() == count)
                    && let Some(Some(first)) = arguments.first()
                {
                    return Ok(Some(*first));
                }
                return Ok(None);
            }
            if *op == SYSREQ_N {
                pushed = Vec::new();
            } else {
                pushed.extend(pushes(*op, args)?);
            }
        }
        Ok(None)
    }

    /// `main`'s case table: each case's first instruction.
    fn switch(&self, entry: i64) -> Result<IndexMap<i64, i64>> {
        for (address, op, args) in self.function(entry) {
            if *op != SWITCH {
                continue;
            }
            let index = self.at.get(&(address + signed(arg(args, 0)?))).copied();
            let Some(index) = index.filter(|&index| self.code[index].1 == CASETBL) else {
                return Ok(IndexMap::new());
            };
            let (table, _op, cells) = &self.code[index];
            let mut out: IndexMap<i64, i64> = IndexMap::new();
            for pair in 0..arg(cells, 0)? {
                // An offset is from the cell before it, the case's value.
                let value_at = table + 4 * (3 + 2 * pair);
                out.insert(
                    signed(arg(cells, (2 + 2 * pair) as usize)?),
                    value_at + signed(arg(cells, (3 + 2 * pair) as usize)?),
                );
            }
            return Ok(out);
        }
        Ok(IndexMap::new())
    }

    /// The function a case calls: `push 0, call`, then the jump out of the switch.
    fn first_call(&self, address: i64) -> Result<Option<i64>> {
        let mut index = self.at.get(&address).copied();
        while let Some(at_index) = index
            && at_index < self.code.len()
        {
            let (at, op, args) = &self.code[at_index];
            if *op == CALL {
                return Ok(Some((at + signed(arg(args, 0)?)) & 0xFFFFFFFF));
            }
            if *op != PUSH_P_C {
                return Ok(None);
            }
            index = Some(at_index + 1);
        }
        Ok(None)
    }

    /// A function's instructions, from its start to the next function's.
    fn function(&self, address: i64) -> &[Instruction] {
        let Some(&index) = self.at.get(&address) else {
            return &[];
        };
        let end = self.code[index..]
            .iter()
            .enumerate()
            .position(|(offset, (_address, op, _args))| *op == PROC && offset != 0)
            .map_or(self.code.len(), |offset| index + offset);
        &self.code[index..end]
    }

    /// Whether a function comes to the message window within `depth` calls.
    fn shows(&self, function: i64, depth: i64) -> Result<bool> {
        if let Some(&known) = self.reaches.borrow().get(&function) {
            return Ok(known);
        }
        self.reaches.borrow_mut().insert(function, false);
        let mut found = false;
        for (address, op, args) in self.function(function) {
            if *op == SYSREQ_N {
                let native = arg(args, 0)?;
                if native < self.natives.len() as i64 && self.natives[native as usize] == MESSAGE {
                    found = true;
                    break;
                }
            }
            if *op == CALL
                && depth > 1
                && self.shows((address + signed(arg(args, 0)?)) & 0xFFFFFFFF, depth - 1)?
            {
                found = true;
                break;
            }
        }
        self.reaches.borrow_mut().insert(function, found);
        Ok(found)
    }
}

/// A call's arguments in order from what was pushed for it, the last push their size in
/// bytes, or None where that is not known.
fn arguments(pushed: &[Option<i64>]) -> Option<Vec<Option<i64>>> {
    let size = (*pushed.last()?)?;
    if py::modulo(size, 4) != 0 {
        return None;
    }
    let count = py::floordiv(size, 4);
    let len = pushed.len() as i64;
    if count > len - 1 {
        return None;
    }
    let mut out = py::slice(pushed, len - 1 - count, -1).to_vec();
    out.reverse();
    Some(out)
}

/// What an instruction pushes, a constant or None for anything else; nothing for an
/// instruction that pushes nothing.
fn pushes(op: i64, args: &[i64]) -> Result<Vec<Option<i64>>> {
    if op == PUSH_P_C {
        return Ok(vec![Some(arg(args, 0)?)]);
    }
    if op == PUSH_C {
        return Ok(vec![Some(signed(arg(args, 0)?))]);
    }
    if push_n_c(op).is_some() {
        return Ok(args.iter().map(|&arg| Some(signed(arg))).collect());
    }
    if [36, 37, 40, 41, 133, 0xBD, 0xBE].contains(&op) || (139..=153).contains(&op) {
        return Ok(vec![None; args.len().max(1)]);
    }
    Ok(Vec::new())
}

/// The image's cells from `start`, out of Pawn's compact encoding.
fn expanded(data: &[u8], start: i64) -> Vec<i64> {
    let mut cells: Vec<i64> = Vec::new();
    let (mut at, end) = (start, data.len() as i64);
    while at < end {
        let mut byte = data[at as usize] as i64;
        // Python's int keeps every bit shifted up; the cell is its low 32, which an i64 keeps.
        let mut cell: i64 = if byte & 0x40 != 0 { -1 } else { 0 };
        loop {
            cell = (cell << 7) | (byte & 0x7F);
            at += 1;
            if byte & 0x80 == 0 || at >= end {
                break;
            }
            byte = data[at as usize] as i64;
        }
        cells.push(cell & 0xFFFFFFFF);
    }
    cells
}

/// Each instruction, by its address from the code's start.
fn decoded(cells: &[i64]) -> Vec<Instruction> {
    let mut out: Vec<Instruction> = Vec::new();
    let mut index: i64 = 0;
    let len = cells.len() as i64;
    while index < len {
        let cell = cells[index as usize];
        let op = cell & 0xFFFF;
        if op >= PACKED {
            out.push((index * 4, op, vec![signed16(cell >> 16)]));
            index += 1;
            continue;
        }
        let size = if op == CASETBL {
            let count = if index + 1 < len {
                cells[(index + 1) as usize]
            } else {
                0
            };
            2 + 2 * count
        } else if let Some(count) = push_n_c(op) {
            count
        } else if (138..=153).contains(&op) {
            (op - 138) / 4 + 2
        } else if TWO_OPERANDS.contains(&op) {
            2
        } else if NO_OPERAND.contains(&op) {
            0
        } else {
            1
        };
        out.push((
            index * 4,
            op,
            py::slice(cells, index + 1, index + 1 + size).to_vec(),
        ));
        index += 1 + size;
    }
    out
}

fn signed(value: i64) -> i64 {
    if value & 0x80000000 != 0 {
        value - (1 << 32)
    } else {
        value
    }
}

fn signed16(value: i64) -> i64 {
    if value & 0x8000 != 0 {
        value - (1 << 16)
    } else {
        value
    }
}

/// An area's 'ZS' pack, a script a zone in the area's order, None where one is not read.
pub fn scripts(pack: &[u8]) -> Result<Vec<Option<ZoneScript>>> {
    if py::slice(pack, 0, 2) != b"ZS" || pack.len() < 8 {
        return Ok(Vec::new());
    }
    let count = u16(pack, 2)?;
    let mut offsets: Vec<i64> = Vec::new();
    if 8 + count * 4 <= pack.len() as i64 {
        for i in 0..=count {
            offsets.push(u32(pack, 4 + 4 * i)?);
        }
    }
    let mut out: Vec<Option<ZoneScript>> = Vec::new();
    for pair in offsets.windows(2) {
        match ZoneScript::new(py::slice(pack, pair[0], pair[1])) {
            Ok(script) => out.push(Some(script)),
            Err(error)
                if error.is(Kind::ValueError)
                    || error.is(Kind::IndexError)
                    || error.is(Kind::StructError) =>
            {
                out.push(None);
            }
            Err(error) => return Err(error),
        }
    }
    Ok(out)
}
