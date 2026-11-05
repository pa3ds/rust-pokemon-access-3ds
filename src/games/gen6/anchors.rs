//! What the running app's process leads to: objects found by a chain of offsets from the process,
//! from the moment the app starts, where a sweep found them seconds on (spike 282).
//!
//! The runtime knows the process of the app running from its first poll (`gen6.apps`), and an app
//! keeps its parts in its process, or a few pointers down. A **chain** follows them: from the
//! process, each hop read as a pointer at that offset into the object before, the last landing on
//! the object. With a `count` and a `stride` it lands on one object of a pool of `count`, `stride`
//! apart, and the pool is the run of the class's objects round it; with a `count` alone the last
//! hop is an array of `count` pointers, read where it lands:
//!
//! ```text
//! the field's  +0x24 -> the field's work  +0x164 -> the models' manager  +0x0c -> one of its 32
//!              move models, the fifth on Route 104, a pool 0xab0 apart in ORAS
//! ```
//!
//! Each object is checked by its class before a reader is handed it, so a chain that lands
//! elsewhere, or before the app has built the object, hands nothing over, and is followed again
//! `poll` by poll: the PlayNav's frame holds a page's object only once the page is built.
//!
//! What a chain finds is handed to the readers as a partial find, as the heaps' watch hands over
//! what it sweeps in new blocks (ADR-0034): a reader adds it to what it has. Only classes whose
//! readers take a partial find so are chained. The sweep goes on looking for every class chained
//! too (Orion, 2026-11-03): a chain is offsets into the game's own objects, measured in Omega Ruby
//! and X, and one that lands elsewhere in a version or a moment not measured hands nothing over,
//! where the sweep finds the object as it did, a second or two later. A chain is only as good as
//! the object it lands on being the one in use: Head It's process leads to a `Model` of the class
//! that is not the round's, and its balls went unnamed, so Amie's games are not chained. The
//! chains are each game's (`Game.anchors`): the field's work keeps its parts at other offsets in
//! X.
//!
//! Bound by:
//! - ADR-0034: what an app keeps is followed from its process, each object checked by its class,
//!   handed over as a partial find, and swept for as well.

use std::collections::HashMap;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::discovery::{Found, HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::gen6::apps::LOADED_CODE;
use crate::py::{self, Result};

/// Objects of `cls` reached from an app's process (module docstring).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chain {
    pub cls: &'static str,
    pub hops: Vec<i64>,
    pub count: i64,
    pub stride: i64,
}

impl Chain {
    /// `Chain(cls, hops)`: one object, `count` 1 and `stride` 0; set them with
    /// `Chain { count, stride, ..Chain::new(cls, hops) }`.
    pub fn new(cls: &'static str, hops: &[i64]) -> Self {
        Chain {
            cls,
            hops: hops.to_vec(),
            count: 1,
            stride: 0,
        }
    }
}

/// Where `hops` lead from `start`, each read as a pointer at that offset into the object
/// before; None where one leaves the heap.
pub fn follow(memory: &dyn Memory, start: i64, hops: &[i64]) -> Result<Option<i64>> {
    let mut at = start;
    for &hop in hops {
        at = u32(&memory.read(at + hop, 4)?, 0)?;
        if !within(HEAP_SPAN, at) {
            return Ok(None);
        }
    }
    Ok(Some(at))
}

/// Follows the chains of the app running, and says what they lead to that is new.
pub struct Anchors {
    memory: Rc<dyn Memory>,
    chains: IndexMap<String, Vec<Chain>>,
    /// What each chain led to when it was last followed, as handed over.
    last: HashMap<Chain, Vec<i64>>,
}

impl Anchors {
    pub fn new(memory: Rc<dyn Memory>, chains: IndexMap<String, Vec<Chain>>) -> Self {
        Anchors {
            memory,
            chains,
            last: HashMap::new(),
        }
    }

    /// Another app has started: what it leads to is all new.
    pub fn started(&mut self) {
        self.last.clear();
    }

    /// What the chains of the app `running` lead to from its `process`, each chain's objects
    /// whenever they are other than they were at its last follow, as a partial find; None where
    /// none has changed. Not only what is new: walking out of a Secret Base, the field's map
    /// blocks went back to the four the route had before it, and a reader that had taken the
    /// base's in between had none of the route's (or_secret_base_objects).
    pub fn poll(&mut self, running: Option<&str>, process: Option<i64>) -> Result<Option<Found>> {
        let chains = self
            .chains
            .get(running.unwrap_or(""))
            .cloned()
            .unwrap_or_default();
        let Some(process) = process else {
            return Ok(None);
        };
        if chains.is_empty() {
            return Ok(None);
        }
        let names = ClassNames::new(self.memory.clone(), LOADED_CODE);
        let mut new: IndexMap<i64, String> = IndexMap::new();
        for chain in chains {
            let found = self.follow(process, &chain, &names)?;
            if found.as_slice() != self.last.get(&chain).map_or(&[][..], Vec::as_slice) {
                // Kept empty too, so the same objects come back as news: the mapper of a base
                // walked into a second time.
                for &at in &found {
                    new.insert(at, chain.cls.to_string());
                }
                self.last.insert(chain, found);
            }
        }
        Ok(if new.is_empty() {
            None
        } else {
            Some(Found {
                pages: IndexMap::new(),
                heap: new,
                partial: true,
            })
        })
    }

    /// The objects of its class a chain leads to; none where a hop leaves the heap.
    fn follow(&self, process: i64, chain: &Chain, names: &ClassNames) -> Result<Vec<i64>> {
        let array = chain.count > 1 && chain.stride == 0;
        let hops = if array {
            py::slice(&chain.hops, 0, -1)
        } else {
            &chain.hops[..]
        };
        let Some(at) = follow(&*self.memory, process, hops)? else {
            return Ok(Vec::new());
        };
        if array {
            let words = self
                .memory
                .read(at + chain.hops[chain.hops.len() - 1], 4 * chain.count)?;
            let mut found = Vec::new();
            for i in 0..chain.count {
                found.push(u32(&words, 4 * i)?);
            }
            let mut out = Vec::new();
            for at in found {
                if within(HEAP_SPAN, at) && names.of_object(at)?.as_deref() == Some(chain.cls) {
                    out.push(at);
                }
            }
            return Ok(out);
        }
        if names.of_object(at)?.as_deref() != Some(chain.cls) {
            return Ok(Vec::new());
        }
        if chain.count == 1 {
            return Ok(vec![at]);
        }
        // A pool: back from the one landed on to the first of the run, and `count` from there,
        // every one of the class, or none.
        let mut first = at;
        while py::floordiv(at - first, chain.stride) < chain.count - 1
            && names.of_object(first - chain.stride)?.as_deref() == Some(chain.cls)
        {
            first -= chain.stride;
        }
        let pool: Vec<i64> = (0..chain.count).map(|i| first + chain.stride * i).collect();
        for &at in &pool {
            if names.of_object(at)?.as_deref() != Some(chain.cls) {
                return Ok(Vec::new());
            }
        }
        Ok(pool)
    }
}
