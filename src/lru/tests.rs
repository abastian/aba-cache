use super::*;

#[test]
#[should_panic]
fn test_create_cache_with_cap_0() {
    Cache::<usize, ()>::new(0);
}

#[test]
fn test_get_on_empty_cache() {
    let mut cache = Cache::<(), usize>::new(1);

    assert_eq!(cache.get(&()).err(), Some(Error::EmptyStorage));
}

#[test]
fn test_get_uncached_key() {
    let mut cache = Cache::<usize, usize>::new(1);

    cache.put(1, 1);

    assert_eq!(cache.get(&2).err(), Some(Error::KeyNotExist));
}

#[test]
fn test_reuse_single_entry() {
    let mut cache = Cache::<usize, usize>::new(1);

    let old_value = cache.put(1, 1);
    assert_eq!(old_value, None);

    let old_value = cache.put(1, 2);
    assert_eq!(old_value, Some(1));
    assert_eq!(cache.map.len(), 1);
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
    assert_eq!(cache.map.len(), 2);
}

#[test]
fn test_get_head_entry() {
    let mut cache = Cache::<usize, &str>::new(2);

    cache.put(1, "one");
    cache.put(2, "two");

    let cache_head = cache.get(&2);
    assert_eq!(cache_head.ok(), Some(&"two"));
}

#[test]
fn test_get_least_entry() {
    let mut cache = Cache::<usize, &str>::new(3);

    cache.put(1, "one");
    cache.put(2, "two");
    cache.put(3, "three");

    let cache_head = cache.get(&1);
    assert_eq!(cache_head.ok(), Some(&"one"));
}

#[test]
fn test_get_middle_entry() {
    let mut cache = Cache::<usize, &str>::new(3);

    cache.put(1, "one");
    cache.put(2, "two");
    cache.put(3, "three");

    let cache_head = cache.get(&2);
    assert_eq!(cache_head.ok(), Some(&"two"));
}
