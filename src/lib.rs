//! In-Process In-Memory Cache implementation.
//!
//! Supported eviction strategy:
//! - LRU (Least Recently Used, inspired by [LRU Cache](https://github.com/jeromefroe/lru-rs))
//! - LFU (Least Frequently Used, TBD)
//! - MRU (Most Recently Used, TBD)
//! - FIFO (First In First Out, TBD)
//!
//! Supporting async/.await powered by tokio runtime.
//!
//! ## Caveat
//!
//! If you need to use asynchronous, enable feature "asynchronous" for this crate on your `Cargo.toml`
//!
//! If you do need to use reference for your value, on non-asynchronous do use `std::rc::Rc`,
//! and on asynchronous do use `std::sync::Arc`
//!
//! ## Example basic
//!
//! ### Cargo.toml
//!
//! ```toml
//! [dependencies]
//! aba-cache = { version = "0.1", default-features = false }
//! serde_json = { version = "1.0" }
//! ```
//!
//! ### main.rs
//!
//! ```
//! use aba_cache as cache;
//! use cache::LruCache;
//! use serde_json::{self, Value};
//! use std::rc::Rc;
//!
//! // create cache with multiply_cap 1024, and evicting
//! // LRU entry with age 15 minutes or older
//! let mut cache = LruCache::new(1024, 15 * 60);
//!
//! let val_one: Rc<Value> = Rc::new(serde_json::from_str(r#"{"id":1}"#).unwrap());
//! let val_two: Rc<Value> = Rc::new(serde_json::from_str(r#"{"id":2}"#).unwrap());
//!
//! cache.put("one", val_one.clone());
//! cache.put("two", val_two.clone());
//!
//! assert!(
//!     if let Some(value) = cache.get(&"one") {
//!         *value == val_one
//!     } else {
//!         false
//!     }
//! );
//! assert_eq!(cache.get(&"three"), None);
//! ```
//!
//! ## Example async
//!
//! ### Cargo.toml
//!
//! ```toml
//! aba-cache = { version = "0.1", default-features = false, features = ["asynchronous"] }
//! serde_json = { version = "1.0" }
//! tokio = { version = "0.2", features = ["macros", "rt-core"] }
//! ```
//!
//! ### main.rs
//!
//! ```
//! use aba_cache as cache;
//! use cache::LruAsyncCache;
//! use serde_json::{self, Value};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // create cache with multiply_cap 1024, and evicting
//!     // LRU entry with age 15 minutes or older
//!     let cache = LruAsyncCache::new(1024, 15 * 60);
//!
//!     let val_one: Arc<Value> = Arc::new(serde_json::from_str(r#"{"id":1}"#)?);
//!     let val_two: Arc<Value> = Arc::new(serde_json::from_str(r#"{"id":2}"#)?);
//!
//!     cache.put("one", val_one.clone()).await;
//!     cache.put("two", val_two.clone()).await;
//!
//!     assert!(if let Some(value) = cache.get(&"one").await {
//!         value == val_one
//!     } else {
//!         false
//!     });
//!     assert_eq!(cache.get(&"three").await, None);
//!
//!     Ok(())
//! }
//! ```
mod lru;

#[cfg(feature = "update-intent")]
pub use lru::asynchronous::update_intent::Cache as LruAsyncUpdateIntentCache;
#[cfg(feature = "asynchronous")]
pub use lru::asynchronous::Cache as LruAsyncCache;
pub use lru::basic::Cache as LruCache;
