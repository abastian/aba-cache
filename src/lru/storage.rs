use slab::Slab;
use std::{
    mem,
    ops::{Index, IndexMut},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(PartialEq, Copy, Clone)]
pub(super) enum Pointer {
    Null,
    InternalPointer { slab: usize, pos: usize },
}

impl Pointer {
    #[inline]
    pub(super) fn null() -> Pointer {
        Pointer::Null
    }

    // Returns `true` if this pointer is null.
    #[inline]
    pub(super) fn is_null(&self) -> bool {
        *self == Pointer::Null
    }
}

pub(super) struct Storage<K, V> {
    slabs: Slab<Slab<Entry<K, V>>>,

    cap: usize,
    len: usize,

    head: Pointer,
    tail: Pointer,

    timeout_secs: u64,
}

pub(super) struct Entry<K, V> {
    key: K,
    timestamp: u64,
    data: V,

    next: Pointer,
    prev: Pointer,
}

impl<K, V> Entry<K, V> {
    fn new(key: K, data: V, next: Pointer, prev: Pointer) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Entry {
            key,
            timestamp,
            data,
            next,
            prev,
        }
    }
}

/// Simplifying read access to elements contained within.
impl<K, V> Index<Pointer> for Storage<K, V> {
    type Output = Entry<K, V>;

    fn index(&self, index: Pointer) -> &Self::Output {
        if let Pointer::InternalPointer { slab, pos } = index {
            self.slabs[slab].index(pos)
        } else {
            panic!("indexing on null pointer");
        }
    }
}

/// Simplifying write access to elements contained within.
impl<K, V> IndexMut<Pointer> for Storage<K, V> {
    fn index_mut(&mut self, index: Pointer) -> &mut Self::Output {
        if let Pointer::InternalPointer { slab, pos } = index {
            self.slabs[slab].index_mut(pos)
        } else {
            panic!("indexing on null pointer");
        }
    }
}

impl<K, V> Storage<K, V> {
    pub(super) fn new(cap: usize, timeout_secs: u64) -> Self {
        let mut slabs = Slab::new();
        slabs.insert(Slab::with_capacity(cap));
        Storage {
            slabs,
            cap,
            len: 0,
            head: Pointer::null(),
            tail: Pointer::null(),
            timeout_secs,
        }
    }

    /// Insert a key-value.
    /// return two data on a tuple
    /// - new index,
    /// - old pair key-value on update case or None on insert
    pub(super) fn put(&mut self, key: K, data: V) -> (Pointer, Option<(K, V)>) {
        if !self.tail.is_null() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let ptr = self.tail;
            // update expired entry
            if self[ptr].timestamp + self.timeout_secs <= now {
                let tail = if self.head == ptr {
                    // single content, already on top
                    &mut self[ptr]
                } else {
                    self.move_to_top(self.tail)
                };
                let old_key = mem::replace(&mut tail.key, key);
                let old_data = mem::replace(&mut tail.data, data);
                tail.timestamp = now;
                return (ptr, Some((old_key, old_data)));
            }
        }

        // allocate new slab if necessary
        let slab = {
            let mut slab_id = None;
            for (key, data) in self.slabs.iter() {
                if data.len() < data.capacity() {
                    slab_id = Some(key);
                    break;
                }
            }
            if let Some(slab_id) = slab_id {
                slab_id
            } else {
                self.slabs.insert(Slab::with_capacity(self.cap))
            }
        };

        // insert entry
        let entry = Entry::new(key, data, self.head, Pointer::null());
        let id = Pointer::InternalPointer {
            slab,
            pos: self.slabs[slab].insert(entry),
        };
        if self.head.is_null() {
            self.tail = id;
        } else {
            let idx = self.head;
            self[idx].prev = id;
        }
        self.head = id;
        self.len += 1;
        (id, None)
    }

    /// Update the data associated with given pointer and move it
    /// to the top of the LRU list, if not already there.
    pub(super) fn update(&mut self, ptr: Pointer, data: V) -> V {
        let top = if self.head == ptr {
            // single content, already on top
            &mut self[ptr]
        } else {
            self.move_to_top(ptr)
        };
        top.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        mem::replace(&mut top.data, data)
    }

    /// Return the data associated with given pointer and move it
    /// to the top of the LRU list, if not already there.
    pub(super) fn get(&mut self, ptr: Pointer) -> Option<&V> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if ptr == self.head {
            // already on top
            self[ptr].timestamp = now;
            Some(&self[ptr].data)
        } else {
            let top = self.move_to_top(ptr);
            top.timestamp = now;
            Some(&top.data)
        }
    }

    pub(super) fn capacity(&self) -> usize {
        self.slabs.iter().map(|(_, slab)| slab.capacity()).sum()
    }

    /// Move entry at pointer to the top of LRU list.
    fn move_to_top(&mut self, ptr: Pointer) -> &mut Entry<K, V> {
        let (next, prev) = {
            let target = &self[ptr];
            (target.next, target.prev)
        };
        let head = self.head;

        if next.is_null() {
            // target is the tail, need to move tail to prev
            self.tail = prev;
        } else {
            self[next].prev = prev;
        }
        self[prev].next = next;
        self[head].prev = ptr;
        self.head = ptr;
        {
            let target = &mut self[ptr];
            target.next = head;
            target.prev = Pointer::null();
            target
        }
    }

    #[cfg(test)]
    pub(super) fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        Iter {
            storage: self,
            current: self.head,
        }
    }
}

#[cfg(test)]
pub(super) struct Iter<'a, K, V> {
    storage: &'a Storage<K, V>,
    current: Pointer,
}

#[cfg(test)]
pub(super) struct IterEntry<'a, K, V> {
    ptr: Pointer,
    entry: &'a Entry<K, V>,
}

#[cfg(test)]
impl<'a, K, V> IterEntry<'a, K, V> {
    pub(super) fn ptr(&self) -> Pointer {
        self.ptr
    }

    pub(super) fn next(&self) -> Pointer {
        self.entry.next
    }

    pub(super) fn prev(&self) -> Pointer {
        self.entry.prev
    }
}

#[cfg(test)]
impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = IterEntry<'a, K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            let ptr = self.current;
            let item = IterEntry {
                ptr,
                entry: &self.storage[ptr],
            };
            self.current = item.entry.next;
            Some(item)
        }
    }
}
