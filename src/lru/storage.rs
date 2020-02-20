use slab::Slab;
use std::mem;

const NULL_SIGIL: usize = !0;

pub(super) struct Storage<K, V> {
    entries: Slab<Entry<K, V>>,

    cap: usize,
    len: usize,

    head: usize,
    tail: usize,
}

struct Entry<K, V> {
    key: K,
    data: V,

    next: usize,
    prev: usize,
}

impl<K, V> Entry<K, V> {
    fn new(key: K, data: V, next: usize, prev: usize) -> Self {
        Entry {
            key,
            data,
            next,
            prev,
        }
    }
}

impl<K, V> Storage<K, V> {
    pub(super) fn new(cap: usize) -> Self {
        Storage {
            entries: Slab::with_capacity(cap),
            cap,
            len: 0,
            head: NULL_SIGIL,
            tail: NULL_SIGIL,
        }
    }

    // Insert a key-value.
    // return
    // - new index,
    // - old pair key-value on update case or None on insert
    pub(super) fn put(&mut self, key: K, data: V) -> (usize, Option<(K, V)>) {
        if self.len < self.cap {
            // there's still room
            let entry = Entry::new(key, data, self.head, NULL_SIGIL);
            let id = self.entries.insert(entry);
            if self.head != NULL_SIGIL {
                self.entries[self.head].prev = id;
            } else {
                self.tail = id;
            }
            self.head = id;
            self.len += 1;
            (id, None)
        } else {
            let id = self.tail;
            let tail = if self.head == id {
                // single content, already on top
                &mut self.entries[id]
            } else {
                unsafe { self.move_to_top(self.tail) }
            };
            let old_key = mem::replace(&mut tail.key, key);
            let old_data = mem::replace(&mut tail.data, data);
            (id, Some((old_key, old_data)))
        }
    }

    pub(super) fn update(&mut self, index: usize, data: V) -> V {
        let tail = if self.head == index {
            // single content, already on top
            self.entries.get_mut(index).unwrap()
        } else {
            unsafe { self.move_to_top(index) }
        };
        mem::replace(&mut tail.data, data)
    }

    pub(super) fn get(&mut self, index: usize) -> Option<&V> {
        // valid range for index
        if self.head == NULL_SIGIL {
            // empty list
            None
        } else if index == self.head {
            self.entries.get(index).map(|entry| &entry.data)
        } else {
            Some(&unsafe { self.move_to_top(index) }.data)
        }
    }

    pub(super) fn capacity(&self) -> usize {
        self.cap
    }

    /// Move entry at index to head position
    ///
    /// # Safety
    ///
    /// It is undefined behaviour to call this function when any pointer
    /// is out of bound.
    unsafe fn move_to_top(&mut self, index: usize) -> &mut Entry<K, V> {
        // need to access elements inside as mutable,
        // on normal condition, rust can't borrow multiple mutable elements
        // on one container at one time
        let target = &mut *(self.entries.get_unchecked_mut(index) as *mut Entry<K, V>);
        let next = if target.next != NULL_SIGIL {
            Some(&mut *(self.entries.get_unchecked_mut(target.next) as *mut Entry<K, V>))
        } else {
            None
        };
        let prev = &mut *(self.entries.get_unchecked_mut(target.prev) as *mut Entry<K, V>);
        let head = &mut *(self.entries.get_unchecked_mut(self.head) as *mut Entry<K, V>);

        if let Some(next) = next {
            next.prev = target.prev;
        } else {
            // target is the tail, need to move tail to prev
            self.tail = target.prev;
        }
        prev.next = target.next;
        target.next = self.head;
        target.prev = NULL_SIGIL;
        head.prev = index;
        self.head = index;
        target
    }

    #[cfg(test)]
    pub(crate) fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        Iter {
            storage: self,
            current: self.head,
        }
    }
}

#[cfg(test)]
pub(crate) struct Iter<'a, K, V> {
    storage: &'a Storage<K, V>,
    current: usize,
}

#[cfg(test)]
pub(crate) struct IterEntry<'a, K, V> {
    id: usize,
    entry: &'a Entry<K, V>,
}

#[cfg(test)]
impl<'a, K, V> IterEntry<'a, K, V> {
    pub(crate) fn id(&self) -> usize {
        self.id
    }

    pub(crate) fn next(&self) -> usize {
        self.entry.next
    }

    pub(crate) fn prev(&self) -> usize {
        self.entry.prev
    }
}

#[cfg(test)]
impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = IterEntry<'a, K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == NULL_SIGIL {
            None
        } else {
            let item = IterEntry {
                id: self.current,
                entry: &self.storage.entries[self.current],
            };
            self.current = item.entry.next;
            Some(item)
        }
    }
}
