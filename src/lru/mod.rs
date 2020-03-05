use std::{borrow::Borrow, collections::HashMap, hash::Hash, rc::Rc};

use storage::{Pointer, Storage};

pub(crate) mod asynchronous;
mod storage;

#[cfg(test)]
mod tests;

pub struct Cache<K, V> {
    storage: Storage<Rc<K>, V>,
    map: HashMap<Rc<K>, Pointer>,
}

impl<K: Hash + Eq, V> Cache<K, V> {
    /// Create new Cache, which will expiring its entry after `timeout_secs`
    /// and allocating new slab with capacity `multiply_cap` when no space
    /// is ready and no entry expires
    pub fn new(multiply_cap: usize, timeout_secs: u64) -> Self {
        if multiply_cap == 0 {
            panic!("Cache defined with 0 capacity")
        }
        Cache {
            storage: Storage::new(multiply_cap, timeout_secs),
            map: HashMap::with_capacity(multiply_cap),
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
    ///
    /// let mut cache = LruCache::new(2, 60);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    /// cache.put(2, "c");
    /// cache.put(3, "d");
    ///
    /// assert_eq!(cache.get(&1), Some(&"a"));
    /// assert_eq!(cache.get(&2), Some(&"c"));
    /// assert_eq!(cache.get(&3), Some(&"d"));
    /// ```
    pub fn get<Q: ?Sized>(&mut self, key: &Q) -> Option<&V>
    where
        Rc<K>: Borrow<Q>,
        Q: Hash + Eq,
    {
        if self.map.is_empty() {
            None
        } else if let Some(&index) = self.map.get(key) {
            let result = self.storage.get(index);
            if result.is_none() {
                self.map.remove(key);
            }
            result
        } else {
            None
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
    /// let mut cache = LruCache::new(2, 60);
    ///
    /// assert_eq!(None, cache.put(1, "a"));
    /// assert_eq!(None, cache.put(2, "b"));
    /// assert_eq!(Some("b"), cache.put(2, "beta"));
    ///
    /// assert_eq!(cache.get(&1), Some(&"a"));
    /// assert_eq!(cache.get(&2), Some(&"beta"));
    /// ```
    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        if let Some(&index) = self.map.get(&key) {
            Some(self.storage.update(index, value))
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

    /// Removes expired entry.
    /// This operation will deallocate empty slab caused by entry removal if any.
    ///
    /// # Example
    ///
    /// ```
    /// use aba_cache as cache;
    /// use cache::LruCache;
    /// use std::{thread, time::Duration};
    ///
    /// let mut cache = LruCache::new(2, 1);
    ///
    /// cache.put(String::from("1"), "one");
    /// cache.put(String::from("2"), "two");
    /// cache.put(String::from("3"), "three");
    ///
    /// assert_eq!(cache.len(), 3);
    /// assert_eq!(cache.capacity(), 4);
    ///
    /// thread::sleep(Duration::from_secs(1));
    /// cache.evict();
    ///
    /// assert_eq!(cache.len(), 0);
    /// assert_eq!(cache.capacity(), 0);
    /// ```
    pub fn evict(&mut self) {
        if !self.is_empty() {
            self.storage.evict().drain(..).for_each(|key| {
                self.map.remove(&key);
            })
        }
    }

    /// Returns the maximum number of key-value pairs the cache can hold.
    /// Note that on data insertion, when no space is available and no
    /// entry is timeout, then capacity will be added with `multiply_cap`
    /// to accomodate.
    ///
    /// # Example
    ///
    /// ```
    /// use aba_cache as cache;
    /// use cache::LruCache;
    ///
    /// let mut cache: LruCache<usize, &str> = LruCache::new(2, 60);
    /// assert_eq!(cache.capacity(), 2);
    ///
    /// cache.put(1, "a");
    /// assert_eq!(cache.capacity(), 2);
    ///
    /// cache.put(2, "b");
    /// assert_eq!(cache.capacity(), 2);
    ///
    /// cache.put(3, "c");
    /// assert_eq!(cache.capacity(), 4);
    /// ```
    pub fn capacity(&self) -> usize {
        self.storage.capacity()
    }

    /// Returns the number of key-value pairs that are currently in the the cache.
    /// Note that len should be less than or equal to capacity
    ///
    /// # Example
    ///
    /// ```
    /// use aba_cache as cache;
    /// use cache::LruCache;
    ///
    /// let mut cache = LruCache::new(2, 60);
    /// assert_eq!(cache.len(), 0);
    ///
    /// cache.put(1, "a");
    /// assert_eq!(cache.len(), 1);
    ///
    /// cache.put(2, "b");
    /// assert_eq!(cache.len(), 2);
    /// assert_eq!(cache.capacity(), 2);
    ///
    /// cache.put(3, "c");
    /// assert_eq!(cache.len(), 3);
    /// assert_eq!(cache.capacity(), 4);
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
    /// let mut cache = LruCache::new(2, 60);
    /// assert!(cache.is_empty());
    ///
    /// cache.put(1, "a");
    /// assert!(!cache.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
