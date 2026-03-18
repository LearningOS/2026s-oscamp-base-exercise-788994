//! # Mutex Shared State
//!
//! In this exercise, you will use `Arc<Mutex<T>>` to safely share and modify data between multiple threads.
//!
//! ## Concepts
//! - `Mutex<T>` mutex protects shared data
//! - `Arc<T>` atomic reference counting enables cross-thread sharing
//! - `lock()` acquires the lock and accesses data

use std::sync::{Arc, Mutex};
use std::thread;

/// Increment a counter concurrently using `n_threads` threads.
/// Each thread increments the counter `count_per_thread` times.
/// Returns the final counter value.
///
/// Hint: Use `Arc<Mutex<usize>>` as the shared counter.
pub fn concurrent_counter(n_threads: usize, count_per_thread: usize) -> usize {
    // TODO: Create Arc<Mutex<usize>> with initial value 0
    // TODO: Spawn n_threads threads
    // TODO: In each thread, lock() and increment count_per_thread times
    // TODO: Join all threads, return final value
    let counter = Arc::new(Mutex::new(0));
    let mut handles = Vec::with_capacity(n_threads);

    // 2. 生成指定数量的线程
    for _ in 0..n_threads {
        // 克隆 Arc（仅增加引用计数，不复制数据）
        let counter_clone = Arc::clone(&counter);
        // 生成线程，在闭包中捕获克隆后的 Arc
        let handle = thread::spawn(move || {
            // 每个线程执行 count_per_thread 次自增
            for _ in 0..count_per_thread {
                // 获取锁（阻塞直到拿到锁），unwrap 处理可能的锁中毒（测试场景下无需复杂处理）
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
                // MutexGuard 会在循环结束时自动释放锁，避免长时间持有
            }
        });
        handles.push(handle);
    }

    // 3. 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 4. 获取最终值并返回
    let final_value = *counter.lock().unwrap();
    final_value
}

/// Add elements to a shared vector concurrently using multiple threads.
/// Each thread pushes its own id (0..n_threads) to the vector.
/// Returns the sorted vector.
///
/// Hint: Use `Arc<Mutex<Vec<usize>>>`.
pub fn concurrent_collect(n_threads: usize) -> Vec<usize> {
    // TODO: Create Arc<Mutex<Vec<usize>>>
    // TODO: Each thread pushes its own id
    // TODO: After joining all threads, sort the result and return
      let shared_vec = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::with_capacity(n_threads);

    // 2. 生成线程，每个线程推送自己的 ID
    for thread_id in 0..n_threads {
        let vec_clone = Arc::clone(&shared_vec);
        let handle = thread::spawn(move || {
            // 获取锁并推送当前线程 ID 到向量
            let mut vec = vec_clone.lock().unwrap();
            vec.push(thread_id);
        });
        handles.push(handle);
    }

    // 3. 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 4. 取出向量并排序后返回
    let mut result = Arc::try_unwrap(shared_vec) // 解开 Arc 包装
        .unwrap() // 处理 Arc 还有其他引用的情况（这里所有线程已结束，无其他引用）
        .into_inner() // 解开 Mutex 包装
        .unwrap(); // 处理锁中毒
    
    result.sort(); // 排序（线程执行顺序不确定，需排序保证结果一致）
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_single_thread() {
        assert_eq!(concurrent_counter(1, 100), 100);
    }

    #[test]
    fn test_counter_multi_thread() {
        assert_eq!(concurrent_counter(10, 100), 1000);
    }

    #[test]
    fn test_counter_zero() {
        assert_eq!(concurrent_counter(5, 0), 0);
    }

    #[test]
    fn test_collect() {
        let result = concurrent_collect(5);
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_collect_single() {
        assert_eq!(concurrent_collect(1), vec![0]);
    }
}
