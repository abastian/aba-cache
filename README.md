# ABA Cache

Simple cache written for reducing frequent access / compute expensive operation (as of now, it serves only as experimental, not intended for production yet)

This implementation is based on "[LRU Cache](https://github.com/jeromefroe/lru-rs)", with another inspiration comes from "[Writing a doubly linked list in Rust is easy](https://www.reddit.com/r/rust/comments/7zsy72/writing_a_doubly_linked_list_in_rust_is_easy/)".

## Example

### Basic LRU Cache

Add following dependencies to Cargo.toml

```toml
[dependencies]
aba-cache = { version = "0.1.0", default-features = false }
```

on your main.rs

```rust
use aba_cache as cache;

fn main() {
    // create Cache, with multiple_cap set to 2
    // and entry will be timeout after 10 seconds
    let mut cache = cache::LruCache::<usize, &str>::new(2, 10);

    cache.put(1, "a");
    cache.put(2, "b");
    cache.put(2, "c");
    cache.put(3, "d");

    assert_eq!(cache.get(&1), Some(&"a"));
    assert_eq!(cache.get(&2), Some(&"c"));
    assert_eq!(cache.get(&3), Some(&"d"));
}
```

### Async LRU Cache

Add following dependencies to Cargo.toml

```toml
[dependencies]
aba-cache = { version = "0.1.0" }
tokio = { version = "0.2", features = ["macros", "rt-core"] }
```

on your main.rs

```rust
use aba_cache as cache;

#[tokio::main]
async fn main() {
    // create Cache, with multiple_cap set to 2
    // and entry will be timeout after 10 seconds
    // additionally, this setup runtime daemon to evict outdate entry
    // every 10 seconds
    let cache = cache::LruAsyncCache::<usize, &str>::new(2, 10);

    cache.put(1, "a").await;
    cache.put(2, "b").await;
    cache.put(2, "c").await;
    cache.put(3, "d").await;

    assert_eq!(cache.get(&1).await, Some("a"));
    assert_eq!(cache.get(&2).await, Some("c"));
    assert_eq!(cache.get(&3).await, Some("d"));
}
```

## References

- [LRU Cache](https://github.com/jeromefroe/lru-rs)
- [Writing a doubly linked list in Rust is easy](https://www.reddit.com/r/rust/comments/7zsy72/writing_a_doubly_linked_list_in_rust_is_easy/)
