mod lru;

#[cfg(feature = "asynchronous")]
pub use lru::asynchronous::Cache as LruAsyncCache;
pub use lru::Cache as LruCache;
