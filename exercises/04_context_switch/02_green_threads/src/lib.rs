//! # Green Thread Scheduler (riscv64)
//!
//! In this exercise, you build a simple cooperative (green) thread scheduler on top of context switching.
//! This crate is **riscv64 only**; run with the repo's normal flow (`./check.sh` / `oscamp`) or natively on riscv64.
//!
//! ## Key Concepts
//! - Cooperative vs preemptive scheduling
//! - Thread state: `Ready`, `Running`, `Finished`
//! - `yield_now()`: current thread voluntarily gives up the CPU
//! - Scheduler loop: pick next ready thread and switch to it
//!
//! ## Design
//! Each green thread has its own stack and `TaskContext`. Threads call `yield_now()` to yield.
//! The scheduler round-robins among ready threads. User entry is wrapped by `thread_wrapper`, which
//! calls the entry then marks the thread `Finished` and switches back.
#![feature(naked_functions)]



#![cfg(target_arch = "riscv64")]

use core::arch::naked_asm;

/// Per-thread stack size. Slightly larger to avoid overflow under QEMU / test harness.
const STACK_SIZE: usize = 1024 * 128;

/// Task context (riscv64); layout must match `01_stack_coroutine::TaskContext` and the asm below.
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct TaskContext {
    sp: u64,
    ra: u64,
    s0: u64,
    s1: u64,
    s2: u64,
    s3: u64,
    s4: u64,
    s5: u64,
    s6: u64,
    s7: u64,
    s8: u64,
    s9: u64,
    s10: u64,
    s11: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThreadState {
    Ready,
    Running,
    Finished,
}

struct GreenThread {
    ctx: TaskContext,
    state: ThreadState,
    _stack: Option<Vec<u8>>,
    /// User entry; taken once when the thread is first scheduled and passed to `thread_wrapper`.
    entry: Option<extern "C" fn()>,
}

/// Set by the scheduler before switching to a new thread; `thread_wrapper` reads and calls it once.
static mut CURRENT_THREAD_ENTRY: Option<extern "C" fn()> = None;

/// Wrapper run as the initial `ra` for each green thread: call the user entry (from `CURRENT_THREAD_ENTRY`), then mark Finished and switch back.
extern "C" fn thread_wrapper() {
    //let entry = unsafe { core::ptr::read(&raw const CURRENT_THREAD_ENTRY) };

    let entry = unsafe { core::ptr::read(core::ptr::addr_of!(CURRENT_THREAD_ENTRY)) };
     

    if let Some(f) = entry {
        unsafe { CURRENT_THREAD_ENTRY = None };
        f();
    }
    thread_finished();
}

/// Save current callee-saved regs into `old`, load from `new`, then `ret` to `new.ra`.
/// Zero `a0`/`a1` before `ret` so we don't leak pointers into the new context.
///
/// Must be `#[unsafe(naked)]` to prevent the compiler from generating a prologue/epilogue.
//#[unsafe(naked)]

#[unsafe(naked)]
//#[naked]
unsafe extern "C" fn switch_context(_old: &mut TaskContext, _new: &TaskContext) {
    naked_asm!(
        "sd sp, 0(a0)",
        "sd ra, 8(a0)",
        "sd s0, 16(a0)",
        "sd s1, 24(a0)",
        "sd s2, 32(a0)",
        "sd s3, 40(a0)",
        "sd s4, 48(a0)",
        "sd s5, 56(a0)",
        "sd s6, 64(a0)",
        "sd s7, 72(a0)",
        "sd s8, 80(a0)",
        "sd s9, 88(a0)",
        "sd s10, 96(a0)",
        "sd s11, 104(a0)",
        "ld sp, 0(a1)",
        "ld ra, 8(a1)",
        "ld s0, 16(a1)",
        "ld s1, 24(a1)",
        "ld s2, 32(a1)",
        "ld s3, 40(a1)",
        "ld s4, 48(a1)",
        "ld s5, 56(a1)",
        "ld s6, 64(a1)",
        "ld s7, 72(a1)",
        "ld s8, 80(a1)",
        "ld s9, 88(a1)",
        "ld s10, 96(a1)",
        "ld s11, 104(a1)",
        "li a0, 0",
        "li a1, 0",
        "ret",
    );
}

pub struct Scheduler {
    threads: Vec<GreenThread>,
    current: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        let main_thread = GreenThread {
            ctx: TaskContext::default(),
            state: ThreadState::Running,
            _stack: None,
            entry: None,
        };

        Self {
            threads: vec![main_thread],
            current: 0,
        }
    }

    /// Register a new green thread that will run `entry` when first scheduled.
    ///
    /// 1. Allocate a stack of `STACK_SIZE` bytes; compute `stack_top` (high address).
    /// 2. Set up the context: `ra = thread_wrapper` so the first switch jumps to the wrapper;
    ///    `sp` must be 16-byte aligned (e.g. `(stack_top - 16) & !15` to leave headroom).
    /// 3. Push a `GreenThread` with this context, state `Ready`, and `entry` stored for the wrapper to call.
    pub fn spawn(&mut self, entry: extern "C" fn()) {
        //todo!("alloc stack, init ctx with ra=thread_wrapper and aligned sp, push GreenThread(Ready, entry)")

        let stack = vec![0u8; STACK_SIZE];
    let stack_top = stack.as_ptr() as usize + STACK_SIZE;
    
    // 2. 初始化上下文
    let mut ctx = TaskContext::default();
    // 关键：设置返回地址为 wrapper，这样第一次 switch_context 就会跳进包装函数
    ctx.ra = thread_wrapper as u64;
    // 关键：栈指针 16 字节对齐
    ctx.sp = (stack_top as u64) & !15;

    // 3. 注册到线程列表
    self.threads.push(GreenThread {
        ctx,
        state: ThreadState::Ready,
        _stack: Some(stack),
        entry: Some(entry), // 给 wrapper 调用的原始函数
    });

    }

    /// Run the scheduler until all threads (except the main one) are `Finished`.
    ///
    /// 1. Set the global `SCHEDULER` pointer to `self` so that `yield_now` and `thread_finished` can call back.
    /// 2. Loop: if all threads in `threads[1..]` are `Finished`, break; otherwise call `schedule_next()` (which may switch away and later return).
    /// 3. Clear `SCHEDULER` when done.
    pub fn run(&mut self) {
        //todo!("set SCHEDULER to self, loop until threads[1..] all Finished, call schedule_next, then clear SCHEDULER")

        unsafe { SCHEDULER = self as *mut Scheduler };

    loop {
        // 2. 检查是否所有用户线程（索引 1 之后）都已经完成
        let all_finished = self.threads[1..].iter().all(|t| t.state == ThreadState::Finished);
        if all_finished {
            break;
        }

        // 3. 尝试切换到下一个任务
        self.schedule_next();
    }

    // 4. 任务结束，清理全局状态
    unsafe { SCHEDULER = std::ptr::null_mut() };

    }

    /// Find the next ready thread (starting from `current + 1` round-robin), mark current as `Ready` (if not `Finished`), mark next as `Running`, set `CURRENT_THREAD_ENTRY` if the next thread has an entry, then switch to it.
    fn schedule_next(&mut self) {
        //todo!("round-robin find next Ready, set current Ready (if not Finished), next Running, CURRENT_THREAD_ENTRY, then switch_context")

        let old_idx = self.current;
    let n_threads = self.threads.len();

    // 1. 寻找下一个处于 Ready 状态的线程
    let mut next_idx = (old_idx + 1) % n_threads;
    while self.threads[next_idx].state != ThreadState::Ready {
        if next_idx == old_idx {
            // 如果找了一圈没找到别的 Ready 线程，直接返回继续跑现在的
            return;
        }
        next_idx = (next_idx + 1) % n_threads;
    }

    // 2. 更新状态
    // 如果当前线程没结束，就把它改回 Ready 以备下次再战
    if self.threads[old_idx].state == ThreadState::Running {
        self.threads[old_idx].state = ThreadState::Ready;
    }
    
    self.threads[next_idx].state = ThreadState::Running;
    self.current = next_idx;

    // 3. 准备跳转环境
    // thread_wrapper 需要通过这个静态变量拿到入口地址
    unsafe { CURRENT_THREAD_ENTRY = self.threads[next_idx].entry.take() };

    // 4. 乾坤大挪移：切换上下文
   // 使用原始指针获取上下文，避开借用检查器的索引限制
unsafe {
    let old_ctx_ptr = &mut self.threads[old_idx].ctx as *mut TaskContext;
    let new_ctx_ptr = &self.threads[next_idx].ctx as *const TaskContext;
    
    // 这里的参数需要根据你的 switch_context 定义来传递
    // 如果 switch_context 接收的是引用，我们需要解引用指针（但在 unsafe 块中）
    switch_context(&mut *old_ctx_ptr, &*new_ctx_ptr);
}


    }
}

impl TaskContext {
    fn as_mut_ptr(&mut self) -> *mut TaskContext {
        self as *mut TaskContext
    }
    fn as_ptr(&self) -> *const TaskContext {
        self as *const TaskContext
    }
}

static mut SCHEDULER: *mut Scheduler = std::ptr::null_mut();

/// Current thread voluntarily yields; the scheduler will pick the next ready thread.
pub fn yield_now() {
    unsafe {
        if !SCHEDULER.is_null() {
            (*SCHEDULER).schedule_next();
        }
    }
}

/// Mark current thread as `Finished` and switch to the next (called by `thread_wrapper` after the user entry returns).
fn thread_finished() {
    unsafe {
        if !SCHEDULER.is_null() {
            let sched = &mut *SCHEDULER;
            sched.threads[sched.current].state = ThreadState::Finished;
            sched.schedule_next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Mutex;

    /// Tests must run serially: the scheduler uses global state (SCHEDULER, CURRENT_THREAD_ENTRY).
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    static EXEC_ORDER: AtomicU32 = AtomicU32::new(0);

    extern "C" fn task_a() {
        EXEC_ORDER.fetch_add(1, Ordering::SeqCst);
        yield_now();
        EXEC_ORDER.fetch_add(10, Ordering::SeqCst);
        yield_now();
        EXEC_ORDER.fetch_add(100, Ordering::SeqCst);
    }

    extern "C" fn task_b() {
        EXEC_ORDER.fetch_add(1, Ordering::SeqCst);
        yield_now();
        EXEC_ORDER.fetch_add(10, Ordering::SeqCst);
    }

    #[test]
    fn test_scheduler_runs_all() {
        let _guard = TEST_LOCK.lock().unwrap();
        EXEC_ORDER.store(0, Ordering::SeqCst);

        let mut sched = Scheduler::new();
        sched.spawn(task_a);
        sched.spawn(task_b);
        sched.run();

        let got = EXEC_ORDER.load(Ordering::SeqCst);
        if got != 122 {
            panic!(
                "EXEC_ORDER: expected 122, got {} (run with --nocapture to see stderr)",
                got
            );
        }
    }

    static SIMPLE_FLAG: AtomicU32 = AtomicU32::new(0);

    extern "C" fn simple_task() {
        SIMPLE_FLAG.store(42, Ordering::SeqCst);
    }

    #[test]
    fn test_single_thread() {
        let _guard = TEST_LOCK.lock().unwrap();
        SIMPLE_FLAG.store(0, Ordering::SeqCst);

        let mut sched = Scheduler::new();
        sched.spawn(simple_task);
        sched.run();

        assert_eq!(SIMPLE_FLAG.load(Ordering::SeqCst), 42);
    }
}
