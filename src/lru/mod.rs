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

    /// Returns a reference to the value of the key in the cache or `None` if it is not
    /// present in the cache. Moves the key to the head of the LRU list if it exists.
    ///
    /// # Example
    ///
    /// ```
    /// use aba_cache as cache;
    /// use cache::LruCache;
    /// use cache::LruError;
    ///
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    /// cache.put(2, "c");
    /// cache.put(3, "d");
    ///
    /// assert_eq!(cache.get(&1).err(), Some(LruError::KeyNotExist));
    /// assert_eq!(cache.get(&2).ok(), Some(&"c"));
    /// assert_eq!(cache.get(&3).ok(), Some(&"d"));
    /// ```
    pub fn get(&mut self, key: &K) -> Result<&V, Error> {
        if self.map.is_empty() {
            Err(Error::EmptyStorage)
        } else if let Some(&index) = self.map.get(key) {
            self.storage.get(index)
        } else {
            Err(Error::KeyNotExist)
        }
    }

    /// Puts a key-value pair into cache. If the key already exists in the cache, then it updates
    /// the key's value and returns the old value. Otherwise, `None` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use aba_cache as cache;
    /// use cache::LruCache;
    ///
    /// let mut cache = LruCache::new(2);
    ///
    /// assert_eq!(None, cache.put(1, "a"));
    /// assert_eq!(None, cache.put(2, "b"));
    /// assert_eq!(Some("b"), cache.put(2, "beta"));
    ///
    /// assert_eq!(cache.get(&1).ok(), Some(&"a"));
    /// assert_eq!(cache.get(&2).ok(), Some(&"beta"));
    /// ```
    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        if let Some(&index) = self.map.get(&key) {
            self.storage.update(index, value).ok()
        } else {
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

    /// Returns the maximum number of key-value pairs the cache can hold.
    ///
    /// # Example
    ///
    /// ```
    /// use aba_cache as cache;
    /// use cache::LruCache;
    ///
    /// let mut cache: LruCache<usize, &str> = LruCache::new(2);
    /// assert_eq!(cache.capacity(), 2);
    /// ```
    pub fn capacity(&self) -> usize {
        self.storage.capacity()
    }

    /// Returns the number of key-value pairs that are currently in the the cache.
    ///
    /// # Example
    ///
    /// ```
    /// use aba_cache as cache;
    /// use cache::LruCache;
    ///
    /// let mut cache = LruCache::new(2);
    /// assert_eq!(cache.len(), 0);
    ///
    /// cache.put(1, "a");
    /// assert_eq!(cache.len(), 1);
    ///
    /// cache.put(2, "b");
    /// assert_eq!(cache.len(), 2);
    ///
    /// cache.put(3, "c");
    /// assert_eq!(cache.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns a bool indicating whether the cache is empty or not.
    ///
    /// # Example
    ///
    /// ```
    /// use aba_cache as cache;
    /// use cache::LruCache;
    ///
    /// let mut cache = LruCache::new(2);
    /// assert!(cache.is_empty());
    ///
    /// cache.put(1, "a");
    /// assert!(!cache.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
