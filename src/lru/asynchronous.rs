use super::Cache as InnerCache;
use std::{borrow::Borrow, hash::Hash, rc::Rc};
use tokio::sync::Mutex;

pub struct Cache<K, V>(Mutex<InnerCache<K, V>>);

impl<K: Hash + Eq, V: Copy + Clone> Cache<K, V> {
    /// Create new Cache, which will expiring its entry after `timeout_secs`
    /// and allocating new slab with capacity `multiply_cap` when no space
    /// is ready and no entry expires.
    pub fn new(multiply_cap: usize, timeout_secs: u64) -> Self {
        Cache(Mutex::new(InnerCache::new(multiply_cap, timeout_secs)))
    }

    /// Returns the value of the key in the cache or `None` if it is not
    /// present in the cache. Moves the key to the head of the LRU list if it exists.
    ///
    /// # Example
    ///
    /// ```
    /// use aba_cache as cache;
    /// use cache::LruAsyncCache;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let cache = LruAsyncCache::new(2, 60);
    ///
    ///     assert_eq!(cache.put(String::from("1"), "a").await, None);
    ///     assert_eq!(cache.put(String::from("2"), "b").await, None);
    ///     assert_eq!(cache.put(String::from("2"), "c").await, Some("b"));
    ///     assert_eq!(cache.put(String::from("3"), "d").await, None);
    ///
    ///     assert_eq!(cache.get(&String::from("1")).await, Some("a"));
    ///     assert_eq!(cache.get(&String::from("2")).await, Some("c"));
    ///     assert_eq!(cache.get(&String::from("3")).await, Some("d"));
    /// }
    /// ```
    pub async fn get<Q: ?Sized>(&self, key: &Q) -> Option<V>
    where
        Rc<K>: Borrow<Q>,
        Q: Hash + Eq,
    {
        let mut cache = self.0.lock().await;
        cache.get(key).cloned()
    }

    /// Puts a key-value pair into cache. If the key already exists in the cache, then it updates
    /// the key's value and returns the old value. Otherwise, `None` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use aba_cache as cache;
    /// use cache::LruAsyncCache;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let cache = LruAsyncCache::new(2, 60);
    ///
    ///     assert_eq!(None, cache.put(String::from("1"), "a").await);
    ///     assert_eq!(None, cache.put(String::from("2"), "b").await);
    ///     assert_eq!(Some("b"), cache.put(String::from("2"), "beta").await);
    ///
    ///     assert_eq!(cache.get(&String::from("1")).await, Some("a"));
    ///     assert_eq!(cache.get(&String::from("2")).await, Some("beta"));
    /// }
    /// ```
    pub async fn put(&self, key: K, value: V) -> Option<V> {
        let mut cache = self.0.lock().await;
        cache.put(key, value)
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
    /// use cache::LruAsyncCache;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let cache = LruAsyncCache::new(2, 60);
    ///     assert_eq!(cache.capacity().await, 2);
    ///
    ///     cache.put(1, "a").await;
    ///     assert_eq!(cache.capacity().await, 2);
    ///
    ///     cache.put(2, "b").await;
    ///     assert_eq!(cache.capacity().await, 2);
    ///
    ///     cache.put(3, "c").await;
    ///     assert_eq!(cache.capacity().await, 4);
    /// }
    /// ```
    pub async fn capacity(&self) -> usize {
        let cache = self.0.lock().await;
        cache.capacity()
    }

    /// Returns the number of key-value pairs that are currently in the the cache.
    /// Note that len should be less than or equal to capacity
    ///
    /// # Example
    ///
    /// ```
    /// use aba_cache as cache;
    /// use cache::LruAsyncCache;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let cache = LruAsyncCache::new(2, 60);
    ///     assert_eq!(cache.len().await, 0);
    ///
    ///     cache.put(1, "a").await;
    ///     assert_eq!(cache.len().await, 1);
    ///
    ///     cache.put(2, "b").await;
    ///     assert_eq!(cache.len().await, 2);
    ///     assert_eq!(cache.capacity().await, 2);
    ///
    ///     cache.put(3, "c").await;
    ///     assert_eq!(cache.len().await, 3);
    ///     assert_eq!(cache.capacity().await, 4);
    /// }
    /// ```
    pub async fn len(&self) -> usize {
        let cache = self.0.lock().await;
        cache.len()
    }

    /// Returns a bool indicating whether the cache is empty or not.
    ///
    /// # Example
    ///
    /// ```
    /// use aba_cache as cache;
    /// use cache::LruAsyncCache;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let cache = LruAsyncCache::new(2, 60);
    ///     assert!(cache.is_empty().await);
    ///
    ///     cache.put(String::from("1"), "a").await;
    ///     assert!(!cache.is_empty().await);
    /// }
    /// ```
    pub async fn is_empty(&self) -> bool {
        let cache = self.0.lock().await;
        cache.is_empty()
    }
}
