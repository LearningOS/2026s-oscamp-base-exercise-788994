//! # Channel Communication
//!
//! In this exercise, you will use `std::sync::mpsc` channels to pass messages between threads.
//!
//! ## Concepts
//! - `mpsc::channel()` creates a multiple producer, single consumer channel
//! - `Sender::send()` sends a message
//! - `Receiver::recv()` receives a message
//! - Multiple producers can be created via `Sender::clone()`

use std::sync::mpsc;
use std::thread;

/// Create a producer thread that sends each element from items into the channel.
/// The main thread receives all messages and returns them.
pub fn simple_send_recv(items: Vec<String>) -> Vec<String> {
    // TODO: Create channel
    // TODO: Spawn thread to send each element in items
    // TODO: In main thread, receive all messages and collect into Vec
    // Hint: When all Senders are dropped, recv() returns Err
  
    // 1. 创建 mpsc 通道
    let (tx, rx) = mpsc::channel();

    // 2. 生成生产者线程，发送所有 items 中的元素
    let handle = thread::spawn(move || {
        for item in items {
            // 发送消息（unwrap 处理发送失败的情况，比如接收端已关闭）
            tx.send(item).unwrap();
        }
        // 线程结束后，tx 会被自动 drop，通道的发送端减少一个引用
    });

    // 3. 主线程接收所有消息
    let mut received = Vec::new();
    // 循环接收，直到所有 Sender 被销毁（recv 返回 Err）
    while let Ok(msg) = rx.recv() {
        received.push(msg);
    }

    // 等待生产者线程完成（可选，但更严谨）
    handle.join().unwrap();

    received
}

/// Create `n_producers` producer threads, each sending a message in format `"msg from {id}"`.
/// Collect all messages, sort them lexicographically, and return.
///
/// Hint: Use `tx.clone()` to create multiple senders. Note that the original tx must also be dropped.
pub fn multi_producer(n_producers: usize) -> Vec<String> {
    // TODO: Create channel
    // TODO: Clone a sender for each producer
    // TODO: Remember to drop the original sender, otherwise receiver won't finish
    // TODO: Collect all messages and sort
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::with_capacity(n_producers);

    // 2. 为每个生产者克隆一个 Sender
    for id in 0..n_producers {
        let tx_clone = tx.clone(); // 克隆发送端
        let handle = thread::spawn(move || {
            // 构造消息并发送
            let msg = format!("msg from {}", id);
            tx_clone.send(msg).unwrap();
        });
        handles.push(handle);
    }

    // 3. 销毁原始的 Sender（关键！否则 rx.recv() 会一直阻塞）
    drop(tx);

    // 4. 接收所有消息
    let mut received = Vec::new();
    while let Ok(msg) = rx.recv() {
        received.push(msg);
    }

    // 等待所有生产者线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 5. 按字典序排序并返回
    received.sort();
    received
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_send_recv() {
        let items = vec!["hello".into(), "world".into(), "rust".into()];
        let result = simple_send_recv(items.clone());
        assert_eq!(result, items);
    }

    #[test]
    fn test_simple_empty() {
        let result = simple_send_recv(vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_multi_producer() {
        let result = multi_producer(3);
        assert_eq!(
            result,
            vec![
                "msg from 0".to_string(),
                "msg from 1".to_string(),
                "msg from 2".to_string(),
            ]
        );
    }

    #[test]
    fn test_multi_producer_single() {
        let result = multi_producer(1);
        assert_eq!(result, vec!["msg from 0".to_string()]);
    }
}
