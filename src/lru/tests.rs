use super::*;
use crate::lru::storage::InternalPointer;

#[test]
#[should_panic]
fn test_create_cache_with_cap_0() {
    Cache::<usize, ()>::new(0);
}

#[test]
fn test_get_on_empty_cache() {
    let mut cache = Cache::<(), usize>::new(1);

    assert!(cache.is_empty());
    assert_eq!(cache.get(&()), None);
}

#[test]
fn test_get_uncached_key() {
    let mut cache = Cache::<usize, usize>::new(1);

    cache.put(1, 1);

    assert_eq!(cache.get(&2), None);
    let mut iter = cache.storage.iter();
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer(Some(InternalPointer { slab: 0, pos: 0 }))
            && item.prev().is_null()
            && item.next().is_null()
    } else {
        false
    });
    assert!(iter.next().is_none());
}

#[test]
fn test_reuse_single_entry() {
    let mut cache = Cache::<usize, usize>::new(1);

    let old_value = cache.put(1, 1);
    assert_eq!(old_value, None);

    let old_value = cache.put(1, 2);
    assert_eq!(old_value, Some(1));
    assert_eq!(cache.len(), 1);
    let mut iter = cache.storage.iter();
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer(Some(InternalPointer { slab: 0, pos: 0 }))
            && item.prev().is_null()
            && item.next().is_null()
    } else {
        false
    });
    assert!(iter.next().is_none());
}

#[test]
fn test_reuse_last_entry() {
    let mut cache = Cache::<usize, usize>::new(2);

    let old_value = cache.put(1, 1);
    assert_eq!(old_value, None);

    let old_value = cache.put(2, 2);
    assert_eq!(old_value, None);

    let old_value = cache.put(3, 3);
    assert_eq!(old_value, Some(1));
    assert_eq!(cache.len(), 2);
    let mut iter = cache.storage.iter();
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer(Some(InternalPointer { slab: 0, pos: 0 }))
            && item.prev().is_null()
            && item.next() == Pointer(Some(InternalPointer { slab: 0, pos: 1 }))
    } else {
        false
    });
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer(Some(InternalPointer { slab: 0, pos: 1 }))
            && item.prev() == Pointer(Some(InternalPointer { slab: 0, pos: 0 }))
            && item.next().is_null()
    } else {
        false
    });
    assert!(iter.next().is_none());
}

#[test]
fn test_get_head_entry() {
    let mut cache = Cache::<usize, &str>::new(2);

    cache.put(1, "one");
    cache.put(2, "two");

    let cache_head = cache.get(&2);
    assert_eq!(cache_head, Some(&"two"));
    let mut iter = cache.storage.iter();
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer(Some(InternalPointer { slab: 0, pos: 1 }))
            && item.prev().is_null()
            && item.next() == Pointer(Some(InternalPointer { slab: 0, pos: 0 }))
    } else {
        false
    });
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer(Some(InternalPointer { slab: 0, pos: 0 }))
            && item.prev() == Pointer(Some(InternalPointer { slab: 0, pos: 1 }))
            && item.next().is_null()
    } else {
        false
    });
    assert!(iter.next().is_none());
}

#[test]
fn test_get_least_entry() {
    let mut cache = Cache::<usize, &str>::new(3);

    cache.put(1, "one");
    cache.put(2, "two");
    cache.put(3, "three");

    let cache_head = cache.get(&1);
    assert_eq!(cache_head, Some(&"one"));
    let mut iter = cache.storage.iter();
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer(Some(InternalPointer { slab: 0, pos: 0 }))
            && item.prev().is_null()
            && item.next() == Pointer(Some(InternalPointer { slab: 0, pos: 2 }))
    } else {
        false
    });
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer(Some(InternalPointer { slab: 0, pos: 2 }))
            && item.prev() == Pointer(Some(InternalPointer { slab: 0, pos: 0 }))
            && item.next() == Pointer(Some(InternalPointer { slab: 0, pos: 1 }))
    } else {
        false
    });
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer(Some(InternalPointer { slab: 0, pos: 1 }))
            && item.prev() == Pointer(Some(InternalPointer { slab: 0, pos: 2 }))
            && item.next().is_null()
    } else {
        false
    });
    assert!(iter.next().is_none());
}

#[test]
fn test_get_middle_entry() {
    let mut cache = Cache::<usize, &str>::new(3);

    cache.put(1, "one");
    cache.put(2, "two");
    cache.put(3, "three");

    let cache_head = cache.get(&2);
    assert_eq!(cache_head, Some(&"two"));
    let mut iter = cache.storage.iter();
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer(Some(InternalPointer { slab: 0, pos: 1 }))
            && item.prev().is_null()
            && item.next() == Pointer(Some(InternalPointer { slab: 0, pos: 2 }))
    } else {
        false
    });
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer(Some(InternalPointer { slab: 0, pos: 2 }))
            && item.prev() == Pointer(Some(InternalPointer { slab: 0, pos: 1 }))
            && item.next() == Pointer(Some(InternalPointer { slab: 0, pos: 0 }))
    } else {
        false
    });
    assert!(if let Some(item) = iter.next() {
        item.ptr() == Pointer(Some(InternalPointer { slab: 0, pos: 0 }))
            && item.prev() == Pointer(Some(InternalPointer { slab: 0, pos: 2 }))
            && item.next().is_null()
    } else {
        false
    });
    assert!(iter.next().is_none());
}
