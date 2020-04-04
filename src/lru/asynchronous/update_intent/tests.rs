use super::*;
use serde_json::{self, Value};
use std::{sync::Arc, time::Duration};
use tokio::time::delay_for;

#[tokio::test]
async fn test_get_expire_entry_async() {
    let cache = Cache::<usize, Arc<Value>>::new(2, 1);

    let val_1: Arc<Value> = Arc::new(serde_json::from_str(r#"{"id":1}"#).unwrap());
    let val_2: Arc<Value> = Arc::new(serde_json::from_str(r#"{"id":2}"#).unwrap());
    let val_3: Arc<Value> = Arc::new(serde_json::from_str(r#"{"id":3}"#).unwrap());

    assert!(cache.put(1, val_1.clone()).await.is_ok());
    assert!(cache.put(2, val_2.clone()).await.is_ok());
    assert!(cache.put(3, val_3.clone()).await.is_ok());

    assert!(if let Some(value) = cache.get(&2).await {
        value == val_2
    } else {
        false
    });

    delay_for(Duration::from_millis(1500)).await;
    assert_eq!(cache.len().await, 0);
    assert_eq!(cache.capacity().await, 0);
}
