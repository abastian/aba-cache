use super::Cache as BaseCache;
use parking_lot::Mutex;
use std::{borrow::Borrow, future::Future, hash::Hash, pin::Pin, sync::Arc};
use tokio::sync::broadcast::{self, Sender};

#[cfg(test)]
mod tests;

pub enum UpdateState<V> {
    Available(V),
    InProgress(Arc<Sender<()>>),
}

pub enum CacheError {
    UpdateInProgress,
}

impl<V: Clone> Clone for UpdateState<V> {
    fn clone(&self) -> Self {
        match self {
            UpdateState::Available(data) => UpdateState::Available(data.clone()),
            UpdateState::InProgress(sender) => UpdateState::InProgress(sender.clone()),
        }
    }
}

pub struct Cache<K, V>(Arc<BaseCache<K, Arc<Mutex<UpdateState<V>>>>>);

impl<K: 'static + Hash + Eq + Sync + Send, V: 'static + Clone + Sync + Send> Cache<K, V> {
    pub fn new(multiply_cap: usize, timeout_secs: u64) -> Arc<Self> {
        let base_cache = BaseCache::new(multiply_cap, timeout_secs);
        Arc::new(Cache(base_cache))
    }

    pub async fn get<Q: ?Sized>(&self, key: &Q) -> Option<V>
    where
        Arc<K>: Borrow<Q>,
        Q: Hash + Eq,
    {
        loop {
            let mut notifier = {
                let mut cache = (self.0).0.lock().await;
                match cache.get(key) {
                    None => return None,
                    Some(value) => match &*value.lock() {
                        UpdateState::Available(v) => return Some(v.clone()),
                        UpdateState::InProgress(sender) => sender.subscribe(),
                    },
                }
            };
            let _ = notifier.recv().await; // ignore error possibility
        }
    }

    pub async fn get_or_update<F>(&mut self, key: K, mut update: F) -> Option<V>
    where
        F: FnMut(&K) -> Pin<Box<dyn Future<Output = V>>>,
    {
        let (fut, sender, data) = loop {
            let mut cache = (self.0).0.lock().await;
            let mut notifier = match cache.get(&key) {
                None => {
                    let fut = update(&key);
                    let (sender, _) = broadcast::channel(1);
                    let sender = Arc::new(sender);
                    let data = Arc::new(Mutex::new(UpdateState::InProgress(sender.clone())));
                    cache.put(key, data.clone());
                    break (fut, sender, data);
                }
                Some(value) => match &*value.lock() {
                    UpdateState::Available(v) => {
                        return Some(v.clone());
                    }
                    UpdateState::InProgress(sender) => sender.subscribe(),
                },
            };
            let _ = notifier.recv().await; // ignore error possibility
        };

        let result = fut.await;
        let mut data = data.lock();
        *data = UpdateState::Available(result.clone());
        let _ = sender.send(()); // ignore error possibility
        Some(result)
    }

    pub async fn put(&self, key: K, value: V) -> Result<Option<V>, CacheError> {
        let mut cache = (self.0).0.lock().await;
        let result = cache
            .get(&key)
            .map(|d| match &*d.lock() {
                UpdateState::Available(v) => Ok(v.clone()),
                UpdateState::InProgress(_) => Err(CacheError::UpdateInProgress),
            })
            .map_or(Ok(None), |res| res.map(Some));

        if result.is_ok() {
            let new_data = Arc::new(Mutex::new(UpdateState::Available(value)));
            cache.put(key, new_data);
        }
        result
    }

    pub async fn capacity(&self) -> usize {
        self.0.capacity().await
    }

    pub async fn len(&self) -> usize {
        self.0.len().await
    }

    pub async fn is_empty(&self) -> bool {
        self.0.is_empty().await
    }
}
