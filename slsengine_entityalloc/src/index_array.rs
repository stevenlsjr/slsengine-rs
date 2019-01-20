use crate::allocator::GenerationalIndex;
use std::{
    fmt,
    ops::{Index, IndexMut},
};

#[derive(Clone, Debug)]
pub struct IndexArray<T>

{
    array: Vec<Option<T>>,
}

impl<T> IndexArray<T>

{
    pub fn new() -> Self {
        IndexArray::with_capacity(256)
    }
    pub fn with_capacity(capacity: usize) -> Self {
        IndexArray {
            array: (0..capacity).map(|_| None).collect(),
        }
    }
    pub fn reserve(&mut self, size: usize) {
        let size = if 16 <= size { size } else { 16 };
        let len = self.array.len();
        self.array.extend((len..size).map(|_| None));
    }

    pub fn insert(&mut self, index: GenerationalIndex, value: T) {
        let i = index.index();
        if self.array.len() <= i {
            self.reserve(i * 2);
        }
        self.array[i] = Some(value);
    }

    pub fn remove(&mut self, index: GenerationalIndex) -> Option<T> {
        use std::mem::replace;
        let i = index.index();
        if self.array.len() <= i {
            None
        } else {
            replace(&mut self.array[i], None)
        }
    }

    pub fn get(&self, index: GenerationalIndex) -> Option<&T> {
        self.index(index).as_ref()
    }

    pub fn get_mut(
        &mut self,
        index: GenerationalIndex,
    ) -> Option<&mut Option<T>> {
        self.array.get_mut(index.index())
    }
}

impl<T> Index<GenerationalIndex> for IndexArray<T>

{
    type Output = Option<T>;
    /// Returns Option<&T> for index. IndexArray does not
    /// maintain a maximum length, and will silenty return None
    /// if index is out of the bounds of internal storage
    fn index(&self, index: GenerationalIndex) -> &Self::Output {
        self.array.get(index.index()).unwrap_or(&None)
    }
}
impl<T> IndexMut<GenerationalIndex> for IndexArray<T>

{
    /// will
    fn index_mut(&mut self, index: GenerationalIndex) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

#[test]
fn test_getters() {
    let mut ie = IndexArray {
        array: vec![Some(1), None, None, Some(2)],
    };
    {
        assert_eq!(ie.get(GenerationalIndex::new(0, 0)), Some(&1));
        assert_eq!(ie.get(GenerationalIndex::new(1, 0)), None);
        assert_eq!(ie.get(GenerationalIndex::new(3, 0)), Some(&2));
    }
    {
        let mut_ptr = ie.get_mut(GenerationalIndex::new(1, 0)).unwrap();
        *mut_ptr = Some(100);
    }
    assert_eq!(ie.get(GenerationalIndex::new(1, 0)), Some(&100));
}

#[test]
fn test_insert() {
    let mut ie: IndexArray<i32> = IndexArray {
        array: vec![None; 1],
    };
    ie.insert(GenerationalIndex::new(10, 0), 10);
    assert_eq!(ie.get(GenerationalIndex::new(10, 0)), Some(&10));
}
