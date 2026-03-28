//! # Async Channel
//!
//! In this exercise, you will use `tokio::sync::mpsc` async channels to implement producer-consumer pattern.
//!
//! ## Concepts
//! - `tokio::sync::mpsc::channel` creates bounded async channels
//! - Async `send` and `recv`
//! - Channel closing mechanism (receiver returns None after all senders are dropped)

use tokio::sync::mpsc;

/// Async producer-consumer:
/// - Create a producer task that sends each element from items sequentially
/// - Create a consumer task that receives all elements and collects them into Vec for return
///
/// Hint: Set channel capacity to items.len().max(1)
pub async fn producer_consumer(items: Vec<String>) -> Vec<String> {
    // TODO: Create channel with mpsc::channel
    // TODO: Spawn producer task: iterate through items, send each one
    // TODO: Spawn consumer task: loop recv until channel closes, collect results
    // TODO: Wait for consumer to complete and return results
    let (tx, mut rx) = mpsc::channel(items.len().max(1));

    // 2. 生成生产者任务
    tokio::spawn(async move {
        for item in items {
            // 发送数据。如果接收端已关闭，则停止发送
            if tx.send(item).await.is_err() {
                break;
            }
        }
    });

    // 3. 生成消费者任务（并返回 JoinHandle 以便稍后等待）
    let consumer_handle = tokio::spawn(async move {
        let mut results = Vec::new();
        // 循环接收直到频道关闭（即所有 tx 被 drop）
        while let Some(msg) = rx.recv().await {
            results.push(msg);
        }
        results
    });

    // 4. 等待消费者任务完成并返回结果
    consumer_handle.await.unwrap()
}

/// Fan‑in pattern: multiple producers, one consumer.
/// Create `n_producers` producers, each sending `"producer {id}: message"`.
/// Consumer collects all messages, sorts them, and returns.
pub async fn fan_in(n_producers: usize) -> Vec<String> {
    // TODO: Create mpsc channel
    // TODO: Spawn n_producers producer tasks
    //       Each sends format!("producer {id}: message")
    // TODO: Drop the original sender (important! otherwise channel won't close)
    let (tx, mut rx) = mpsc::channel(32);

    // 2. 产生 n 个生产者
    for id in 0..n_producers {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let msg = format!("producer {}: message", id);
            let _ = tx_clone.send(msg).await;
        });
    }

    // 3. 重要：必须销毁原始的 tx。
    // 因为 loop 中使用的是 tx.clone()，而这个原始 tx 如果留在作用域内，
    // rx.recv() 就永远等不到“所有发送者已关闭”的信号。
    drop(tx);

    // 4. 消费者收集数据
    let mut results = Vec::new();
    while let Some(msg) = rx.recv().await {
        results.push(msg);
    }

    // 5. 排序并返回
    results.sort();
    results
    
    
    // TODO: Consumer loops receiving, collects and sorts
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_producer_consumer() {
        let items = vec!["hello".into(), "async".into(), "world".into()];
        let result = producer_consumer(items.clone()).await;
        assert_eq!(result, items);
    }

    #[tokio::test]
    async fn test_producer_consumer_empty() {
        let result = producer_consumer(vec![]).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_fan_in() {
        let result = fan_in(3).await;
        assert_eq!(
            result,
            vec![
                "producer 0: message",
                "producer 1: message",
                "producer 2: message",
            ]
        );
    }

    #[tokio::test]
    async fn test_fan_in_single() {
        let result = fan_in(1).await;
        assert_eq!(result, vec!["producer 0: message"]);
    }
}
