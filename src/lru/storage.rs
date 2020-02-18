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
                unsafe {
                    let tail = &mut *(self.entries.get_unchecked_mut(id) as *mut Entry<K, V>);
                    let prev =
                        &mut *(self.entries.get_unchecked_mut(tail.prev) as *mut Entry<K, V>);
                    let head =
                        &mut *(self.entries.get_unchecked_mut(self.head) as *mut Entry<K, V>);
                    prev.next = NULL_SIGIL;
                    head.prev = tail.id;
                    tail.next = head.id;
                    self.tail = prev.id;
                    self.head = tail.id;
                    tail
                }
            };
            let old_key = mem::replace(&mut tail.key, key);
            let old_data = mem::replace(&mut tail.data, data);
            (id, Some((old_key, old_data)))
        }
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
                if self.head >= valid_limit {
                    unreachable!();
                }
                // need to access elements inside as mutable,
                // on normal condition, rust can't borrow multiple mutable elements
                // on one container at one time
                unsafe {
                    let res = &mut *(self.entries.get_unchecked_mut(index) as *mut Entry<K, V>);
                    if res.next >= valid_limit && res.next != NULL_SIGIL {
                        return Err(Error::InvalidIndex);
                    }
                    if res.prev >= valid_limit {
                        return Err(Error::InvalidIndex);
                    }
                    let next = if res.next != NULL_SIGIL {
                        Some(&mut *(self.entries.get_unchecked_mut(res.next) as *mut Entry<K, V>))
                    } else {
                        None
                    };
                    let prev = &mut *(self.entries.get_unchecked_mut(res.prev) as *mut Entry<K, V>);
                    let head =
                        &mut *(self.entries.get_unchecked_mut(self.head) as *mut Entry<K, V>);

                    if let Some(next) = next {
                        next.prev = res.prev;
                    } else {
                        // res is the tail, need to move tail to prev
                        self.tail = res.prev;
                    }
                    prev.next = res.next;
                    res.next = self.head;
                    res.prev = NULL_SIGIL;
                    head.prev = res.id;
                    self.head = res.id;
                    Ok(&res.data)
                }
            }
        } else {
            Err(Error::InvalidIndex)
        }
    }
}
