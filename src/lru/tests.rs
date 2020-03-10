use super::*;
#[cfg(feature = "asynchronous")]
use crate::LruAsyncCache;
use crate::LruCache;
use serde_json::{self, Value};
#[cfg(feature = "asynchronous")]
use std::sync::Arc;
use std::{rc::Rc, thread, time::Duration};
#[cfg(feature = "asynchronous")]
use tokio::time::delay_for;

#[test]
#[should_panic]
fn test_create_cache_with_cap_0() {
    LruCache::<usize, ()>::new(0, 60);
}

#[test]
fn test_get_on_empty_cache() {
    let mut cache = LruCache::<(), usize>::new(1, 60);

    assert!(cache.is_empty());
    assert_eq!(cache.get(&()), None);
}

#[test]
fn test_get_uncached_key() {
    let mut cache = LruCache::<usize, usize>::new(1, 60);

    cache.put(1, 1);

    assert_eq!(cache.get(&2), None);
    let mut iter = cache.storage.iter();
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer::InternalPointer { slab: 0, pos: 0 }
            && item.prev().is_null()
            && item.next().is_null()
    } else {
        false
    });
    assert!(iter.next().is_none());
}

#[test]
fn test_reuse_single_entry() {
    let mut cache = LruCache::<&str, Rc<Value>>::new(1, 60);

    let val_1: Rc<Value> = Rc::new(serde_json::from_str(r#"{"id":1}"#).unwrap());
    let val_2: Rc<Value> = Rc::new(serde_json::from_str(r#"{"id":2}"#).unwrap());

    let old_value = cache.put("1", val_1.clone());
    assert_eq!(old_value, None);

    let old_value = cache.put("1", val_2.clone());
    assert!(if let Some(value) = old_value {
        value == val_1
    } else {
        false
    });
    assert_eq!(cache.len(), 1);
    let mut iter = cache.storage.iter();
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer::InternalPointer { slab: 0, pos: 0 }
            && item.prev().is_null()
            && item.next().is_null()
    } else {
        false
    });
    assert!(iter.next().is_none());
}

#[test]
fn test_reuse_expire_entry() {
    let mut cache = LruCache::<usize, Rc<Value>>::new(2, 1);

    let val_1: Rc<Value> = Rc::new(serde_json::from_str(r#"{"id":1}"#).unwrap());
    let val_2: Rc<Value> = Rc::new(serde_json::from_str(r#"{"id":2}"#).unwrap());

    let old_value = cache.put(1, val_1.clone());
    assert_eq!(old_value, None);

    thread::sleep(Duration::from_secs(1));
    let old_value = cache.put(2, val_2.clone());
    assert!(if let Some(value) = old_value {
        value == val_1
    } else {
        false
    });
    assert_eq!(cache.len(), 1);
    let mut iter = cache.storage.iter();
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer::InternalPointer { slab: 0, pos: 0 }
            && item.prev().is_null()
            && item.next().is_null()
    } else {
        false
    });
    assert!(iter.next().is_none());
}

#[test]
fn test_reuse_last_expire_entry() {
    let mut cache = LruCache::<usize, Rc<Value>>::new(2, 1);

    let val_1: Rc<Value> = Rc::new(serde_json::from_str(r#"{"id":1}"#).unwrap());
    let val_2: Rc<Value> = Rc::new(serde_json::from_str(r#"{"id":2}"#).unwrap());
    let val_3: Rc<Value> = Rc::new(serde_json::from_str(r#"{"id":3}"#).unwrap());

    let old_value = cache.put(1, val_1.clone());
    assert_eq!(old_value, None);

    let old_value = cache.put(2, val_2.clone());
    assert_eq!(old_value, None);

    thread::sleep(Duration::from_secs(1));
    let old_value = cache.put(3, val_3.clone());
    assert!(if let Some(value) = old_value {
        value == val_1
    } else {
        false
    });
    assert_eq!(cache.len(), 2);
    let mut iter = cache.storage.iter();
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer::InternalPointer { slab: 0, pos: 0 }
            && item.prev().is_null()
            && item.next() == Pointer::InternalPointer { slab: 0, pos: 1 }
    } else {
        false
    });
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer::InternalPointer { slab: 0, pos: 1 }
            && item.prev() == Pointer::InternalPointer { slab: 0, pos: 0 }
            && item.next().is_null()
    } else {
        false
    });
    assert!(iter.next().is_none());
}

#[test]
fn test_get_head_entry() {
    let mut cache = LruCache::<usize, &str>::new(2, 60);

    cache.put(1, "one");
    cache.put(2, "two");

    let cache_head = cache.get(&2);
    assert_eq!(cache_head, Some(&"two"));
    let mut iter = cache.storage.iter();
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer::InternalPointer { slab: 0, pos: 1 }
            && item.prev().is_null()
            && item.next() == Pointer::InternalPointer { slab: 0, pos: 0 }
    } else {
        false
    });
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer::InternalPointer { slab: 0, pos: 0 }
            && item.prev() == Pointer::InternalPointer { slab: 0, pos: 1 }
            && item.next().is_null()
    } else {
        false
    });
    assert!(iter.next().is_none());
}

#[test]
fn test_get_least_entry() {
    let mut cache = LruCache::<usize, &str>::new(3, 60);

    cache.put(1, "one");
    cache.put(2, "two");
    cache.put(3, "three");

    let cache_head = cache.get(&1);
    assert_eq!(cache_head, Some(&"one"));
    let mut iter = cache.storage.iter();
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer::InternalPointer { slab: 0, pos: 0 }
            && item.prev().is_null()
            && item.next() == Pointer::InternalPointer { slab: 0, pos: 2 }
    } else {
        false
    });
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer::InternalPointer { slab: 0, pos: 2 }
            && item.prev() == Pointer::InternalPointer { slab: 0, pos: 0 }
            && item.next() == Pointer::InternalPointer { slab: 0, pos: 1 }
    } else {
        false
    });
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer::InternalPointer { slab: 0, pos: 1 }
            && item.prev() == Pointer::InternalPointer { slab: 0, pos: 2 }
            && item.next().is_null()
    } else {
        false
    });
    assert!(iter.next().is_none());
}

#[test]
fn test_get_middle_entry() {
    let mut cache = LruCache::<usize, &str>::new(3, 60);

    cache.put(1, "one");
    cache.put(2, "two");
    cache.put(3, "three");

    let cache_head = cache.get(&2);
    assert_eq!(cache_head, Some(&"two"));
    let mut iter = cache.storage.iter();
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer::InternalPointer { slab: 0, pos: 1 }
            && item.prev().is_null()
            && item.next() == Pointer::InternalPointer { slab: 0, pos: 2 }
    } else {
        false
    });
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer::InternalPointer { slab: 0, pos: 2 }
            && item.prev() == Pointer::InternalPointer { slab: 0, pos: 1 }
            && item.next() == Pointer::InternalPointer { slab: 0, pos: 0 }
    } else {
        false
    });
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer::InternalPointer { slab: 0, pos: 0 }
            && item.prev() == Pointer::InternalPointer { slab: 0, pos: 2 }
            && item.next().is_null()
    } else {
        false
    });
    assert!(iter.next().is_none());
}

#[test]
fn test_get_expire_entry() {
    let mut cache = LruCache::<usize, &str>::new(2, 1);

    cache.put(1, "one");
    cache.put(2, "two");
    cache.put(3, "three");

    let cache_head = cache.get(&2);
    assert_eq!(cache_head, Some(&"two"));

    thread::sleep(Duration::from_secs(1));
    assert_eq!(cache.get(&2), None);
    let mut iter = cache.storage.iter();
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer::InternalPointer { slab: 1, pos: 0 }
            && item.prev().is_null()
            && item.next() == Pointer::InternalPointer { slab: 0, pos: 0 }
    } else {
        false
    });
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer::InternalPointer { slab: 0, pos: 0 }
            && item.prev() == Pointer::InternalPointer { slab: 1, pos: 0 }
            && item.next().is_null()
    } else {
        false
    });
    assert!(iter.next().is_none());
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.capacity(), 4);
}

#[cfg(feature = "asynchronous")]
#[tokio::test]
async fn test_get_expire_entry_async() {
    let cache = LruAsyncCache::<usize, Arc<Value>>::new(2, 1);

    let val_1: Arc<Value> = Arc::new(serde_json::from_str(r#"{"id":1}"#).unwrap());
    let val_2: Arc<Value> = Arc::new(serde_json::from_str(r#"{"id":2}"#).unwrap());
    let val_3: Arc<Value> = Arc::new(serde_json::from_str(r#"{"id":3}"#).unwrap());

    cache.put(1, val_1.clone()).await;
    cache.put(2, val_2.clone()).await;
    cache.put(3, val_3.clone()).await;

    assert!(if let Some(value) = cache.get(&2).await {
        value == val_2
    } else {
        false
    });

    delay_for(Duration::from_millis(1500)).await;
    assert_eq!(cache.len().await, 0);
    assert_eq!(cache.capacity().await, 0);
}
