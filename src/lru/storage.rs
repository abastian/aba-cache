use std::mem;

const NULL_SIGIL: usize = !0;

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyStorage,
    FullStorage,
    InvalidIndex,
    KeyNotExist,
}

pub(super) struct Storage<K, V> {
    entries: Vec<Entry<K, V>>,

    cap: usize,
    next_id: usize,

    head: usize,
    tail: usize,
}

struct Entry<K, V> {
    id: usize,
    key: K,
    data: V,

    next: usize,
    prev: usize,
}

impl<K, V> Entry<K, V> {
    fn new(id: usize, key: K, data: V, next: usize, prev: usize) -> Self {
        Entry {
            id,
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
            entries: Vec::with_capacity(cap),
            cap,
            next_id: 0,
            head: NULL_SIGIL,
            tail: NULL_SIGIL,
        }
    }

    // Insert a key-value.
    // return
    // - new index,
    // - old pair key-value on update case or None on insert
    pub(super) fn put(&mut self, key: K, data: V) -> (usize, Option<(K, V)>) {
        if self.next_id < self.cap {
            // there's still room
            let id = self.next_id;
            self.next_id += 1;
            let entry = Entry::new(id, key, data, self.head, NULL_SIGIL);
            self.entries.insert(id, entry);
            if self.head != NULL_SIGIL {
                self.entries[self.head].prev = id;
            } else {
                self.tail = id;
            }
            self.head = id;
            (id, None)
        } else {
            let id = self.tail;
            let tail = if self.head == id {
                // single content, already on top
                self.entries.get_mut(id).unwrap()
            } else {
                self.move_to_top(self.tail).unwrap()
            };
            let old_key = mem::replace(&mut tail.key, key);
            let old_data = mem::replace(&mut tail.data, data);
            (id, Some((old_key, old_data)))
        }
    }

    pub(super) fn update(&mut self, index: usize, data: V) -> Result<V, Error> {
        let tail = if self.head == index {
            // single content, already on top
            self.entries.get_mut(index).unwrap()
        } else {
            self.move_to_top(index)?
        };
        let old_data = mem::replace(&mut tail.data, data);
        Ok(old_data)
    }

    pub(super) fn get(&mut self, index: usize) -> Result<&V, Error> {
        let valid_limit = self.entries.len();
        if index < valid_limit {
            // valid range for index
            if self.head == NULL_SIGIL {
                // empty list
                Err(Error::EmptyStorage)
            } else if index == self.head {
                Ok(&self.entries.get(index).unwrap().data)
            } else {
                let res = self.move_to_top(index)?;
                Ok(&res.data)
            }
        } else {
            Err(Error::InvalidIndex)
        }
    }

    pub(super) fn capacity(&self) -> usize {
        self.cap
    }

    fn move_to_top(&mut self, index: usize) -> Result<&mut Entry<K, V>, Error> {
        let valid_limit = self.entries.len();
        if self.head >= valid_limit {
            unreachable!();
        }

        // need to access elements inside as mutable,
        // on normal condition, rust can't borrow multiple mutable elements
        // on one container at one time
        let result = unsafe {
            let target = &mut *(self.entries.get_unchecked_mut(index) as *mut Entry<K, V>);
            if target.next >= valid_limit && target.next != NULL_SIGIL {
                return Err(Error::InvalidIndex);
            }
            if target.prev >= valid_limit {
                return Err(Error::InvalidIndex);
            }
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
            head.prev = target.id;
            self.head = target.id;
            target
        };
        Ok(result)
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
pub(crate) struct IterEntry<'a, K, V>(&'a Entry<K, V>);

#[cfg(test)]
impl<'a, K, V> IterEntry<'a, K, V> {
    pub(crate) fn id(&self) -> usize {
        self.0.id
    }

    pub(crate) fn next(&self) -> usize {
        self.0.next
    }

    pub(crate) fn prev(&self) -> usize {
        self.0.prev
    }
}

#[cfg(test)]
impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = IterEntry<'a, K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == NULL_SIGIL {
            None
        } else {
            let item = IterEntry(&self.storage.entries[self.current]);
            self.current = item.0.next;
            Some(item)
        }
    }
}
