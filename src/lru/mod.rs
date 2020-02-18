use std::{collections::HashMap, hash::Hash, rc::Rc};

mod storage;

pub use storage::Error;
use storage::Storage;

#[cfg(test)]
mod tests;

pub struct Cache<K, V> {
    storage: Storage<Rc<K>, V>,
    map: HashMap<Rc<K>, usize>,
}

impl<K: Hash + Eq, V> Cache<K, V> {
    pub fn new(cap: usize) -> Self {
        if cap == 0 {
            panic!("Cache defined with 0 capacity")
        }
        Cache {
            storage: Storage::new(cap),
            map: HashMap::with_capacity(cap),
        }
    }

    pub fn get(&mut self, key: &K) -> Result<&V, Error> {
        if self.map.is_empty() {
            Err(Error::EmptyStorage)
        } else if let Some(&index) = self.map.get(key) {
            self.storage.get(index)
        } else {
            Err(Error::KeyNotExist)
        }
    }

    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        let key = Rc::new(key);
        let (idx, old_pair) = self.storage.put(key.clone(), value);
        let result = if let Some((old_key, old_data)) = old_pair {
            self.map.remove(&old_key);
            Some(old_data)
        } else {
            None
        };
        self.map.insert(key, idx);
        result
    }
}
