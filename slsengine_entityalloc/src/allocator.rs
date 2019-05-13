use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(
    Clone, Copy, Debug, PartialEq, PartialOrd, Hash, Serialize, Deserialize,
)]
pub struct GenerationalIndex {
    index: usize,
    generation: u64,
}

#[wasm_bindgen]
impl GenerationalIndex {
    /// Constructor uses primarily for mocking indices
    /// in a test. Otherwise, indices are created by an allocator
    #[cfg(test)]
    fn new(index: usize, generation: u64) -> Self {
        GenerationalIndex { index, generation }
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
struct AllocEntry {
    is_live: bool,
    generation: u64,
}

/// Maintains a list of generations and free indices
/// Allocates and deallocates indices

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct GenerationalIndexAllocator {
    entries: Vec<AllocEntry>,
    free_list: VecDeque<usize>,
}

#[wasm_bindgen]
impl GenerationalIndexAllocator {
    #[wasm_bindgen(constructor)]
    pub fn with_capacity(capacity: usize) -> Self {
        let entries = vec![
            AllocEntry {
                is_live: false,
                generation: 0,
            };
            capacity
        ];
        let mut free_list: VecDeque<_> = (0usize..capacity).collect();
        free_list.reserve(capacity * 2);
        GenerationalIndexAllocator { entries, free_list }
    }

    fn try_allocate(&mut self) -> Option<GenerationalIndex> {
        self.free_list.pop_front().map(|index| {
            let e = &mut self.entries[index];
            e.is_live = true;
            GenerationalIndex {
                index,
                generation: e.generation,
            }
        })
    }

    pub fn allocate(&mut self) -> GenerationalIndex {
        match self.try_allocate() {
            Some(index) => index,
            None => {
                let mut new_size = self.capacity() * 2;
                if new_size < 16 {
                    new_size = 16;
                }
                self.reserve(new_size);
                self.try_allocate().unwrap()
            }
        }
    }

    pub fn reserve(&mut self, size: usize) {
        let cap = self.capacity();
        if size < cap {
            return;
        }
        self.free_list.extend(cap..size);
        self.entries.extend((cap..size).map(|_| AllocEntry {
            is_live: false,
            generation: 0,
        }));
    }

    // Returns true if the index was allocated before and is now deallocated
    pub fn deallocate(&mut self, index: GenerationalIndex) -> bool {
        if !self.is_live(index) {
            return false;
        }

        let e = &mut self.entries[index.index()];
        e.is_live = false;
        e.generation += 1;

        self.free_list.push_back(index.index());

        true
    }

    pub fn capacity(&self) -> usize {
        self.entries.len()
    }

    pub fn free_capacity(&self) -> usize {
        self.free_list.len()
    }
    pub fn is_live(&self, index: GenerationalIndex) -> bool {
        let e = self.entries[index.index()];
        e.is_live && e.generation == index.generation
    }
}

impl GenerationalIndexAllocator {
    /// Produces an iterator of live entry indices. In the context of a game
    /// ecs, this would iterate through in-scene entities
    pub fn iter_live(&self) -> GenerationalIndexIter {
        GenerationalIndexIter {
            allocator: self,
            begin: 0,
            end: self.entries.len(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GenerationalIndexIter<'a> {
    allocator: &'a GenerationalIndexAllocator,
    begin: usize,
    end: usize,
}

impl<'a> Iterator for GenerationalIndexIter<'a> {
    type Item = GenerationalIndex;
    fn next(&mut self) -> Option<Self::Item> {
        let mut i = self.begin;
        loop {
            if self.end <= i {
                return None;
            }
            let e = self
                .allocator
                .entries
                .get(i)
                .unwrap_or_else(|| panic!("bounds error"));
            i += 1;
            if e.is_live {
                self.begin = i;
                return Some(GenerationalIndex {
                    index: i - 1,
                    generation: e.generation,
                });
            }
        }
    }
}

#[test]
fn test_alloc_entity() {
    let mut gia = GenerationalIndexAllocator::with_capacity(10);
    let entity = gia.allocate();
    assert!(gia.is_live(entity));
    assert_eq!(gia.capacity(), 10);
    assert_eq!(gia.free_capacity(), 9);
}

#[test]
fn test_dealloc_entity() {
    let mut gia = GenerationalIndexAllocator::with_capacity(1);
    let entity = gia.allocate();
    let old_gen = entity.generation;
    assert!(gia.is_live(entity));
    assert!(gia.deallocate(entity));
    assert!(!gia.is_live(entity));
    assert!(!gia.deallocate(entity));
    let new_generation = gia.allocate();
    assert_eq!(new_generation.generation, old_gen + 1);
}

#[test]
fn test_overflow() {
    let mut gia = GenerationalIndexAllocator::with_capacity(1);
    let entity = gia.allocate();
    assert_eq!(gia.capacity(), 1);
    let entity = gia.allocate();
    assert_ne!(gia.capacity(), 1);
}

#[test]
fn test_reserve() {
    let mut gia = GenerationalIndexAllocator::with_capacity(0);
    gia.reserve(10);
    assert_eq!(gia.capacity(), 10);
    gia.reserve(3);
    assert_ne!(
        gia.capacity(),
        3,
        "GenerationalIndexAllocator should never decrease in size"
    );
}

#[test]
fn test_iter() {
    let mut gia = GenerationalIndexAllocator::with_capacity(0);
    gia.allocate();
    gia.allocate();
    gia.allocate();
    let last = gia.allocate();
    gia.deallocate(last);
    let mut iter = gia.iter_live();
    assert!(iter.next().is_some());
    assert!(iter.next().is_some());
    assert!(iter.next().is_some());
    assert!(
        iter.next().is_none(),
        "there should only be 3 live entities"
    );

    assert_eq!(gia.iter_live().count(), 3);
}
