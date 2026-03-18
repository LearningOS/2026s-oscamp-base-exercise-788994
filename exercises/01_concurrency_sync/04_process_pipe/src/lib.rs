//! # Process and Pipes
//!
//! In this exercise, you will learn how to create child processes and communicate through pipes.
//!
//! ## Concepts
//! - `std::process::Command` creates child processes (corresponds to `fork()` + `execve()` system calls)
//! - `Stdio::piped()` sets up pipes (corresponds to `pipe()` + `dup2()` system calls)
//! - Communicate with child processes via stdin/stdout
//! - Obtain child process exit status (corresponds to `waitpid()` system call)
//!
//! ## OS Concepts Mapping
//! This exercise demonstrates user‑space abstractions over underlying OS primitives:
//! - **Process creation**: Rust's `Command::new()` internally invokes `fork()` to create a child process,
//!   then `execve()` (or equivalent) to replace the child's memory image with the target program.
//! - **Inter‑process communication (IPC)**: Pipes are kernel‑managed buffers that allow one‑way data
//!   flow between related processes. The `pipe()` system call creates a pipe, returning two file
//!   descriptors (read end, write end). `dup2()` duplicates a file descriptor, enabling redirection
//!   of standard input/output.
//! - **Resource management**: File descriptors (including pipe ends) are automatically closed when
//!   their Rust `Stdio` objects are dropped, preventing resource leaks.
//!
//! ## Exercise Structure
//! 1. **Basic command execution** (`run_command`) – launch a child process and capture its stdout.
//! 2. **Bidirectional pipe communication** (`pipe_through_cat`) – send data to a child process (`cat`)
//!    and read its output.
//! 3. **Exit code retrieval** (`get_exit_code`) – obtain the termination status of a child process.
//! 4. **Advanced: error‑handling version** (`run_command_with_result`) – learn proper error propagation.
//! 5. **Advanced: complex bidirectional communication** (`pipe_through_grep`) – interact with a filter
//!    program that reads multiple lines and produces filtered output.
//!
//! Each function includes a `TODO` comment indicating where you need to write code.
//! Run `cargo test` to check your implementations.

// use std::io::{self, Read, Write};
// use std::process::{Command, Stdio};


use std::io::{self, BufRead, BufReader, Read, Write};  // 新增 BufRead、BufReader
use std::process::{Command, ExitStatus, Stdio};         // 新增 ExitStatus


/// Execute the given shell command and return its stdout output.
///
/// For example: `run_command("echo", &["hello"])` should return `"hello\n"`
///
/// # Underlying System Calls
/// - `Command::new(program)` → `fork()` + `execve()` family
/// - `Stdio::piped()` → `pipe()` + `dup2()` (sets up a pipe for stdout)
/// - `.output()` → `waitpid()` (waits for child process termination)
///
/// # Implementation Steps
/// 1. Create a `Command` with the given program and arguments.
/// 2. Set `.stdout(Stdio::piped())` to capture the child's stdout.
/// 3. Call `.output()` to execute the child and obtain its `Output`.
/// 4. Convert the `stdout` field (a `Vec<u8>`) into a `String`.
pub fn run_command(program: &str, args: &[&str]) -> String {
    // TODO: Use Command::new to create process
    // TODO: Set stdout to Stdio::piped()
    // TODO: Execute with .output() and get output
    // TODO: Convert stdout to String and return
    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped()) // 2. 重定向stdout到管道
        .output() // 3. 执行命令并获取输出
        .expect("Failed to execute command");

    // 4. 将stdout的字节数组转为字符串（unwrap忽略UTF8错误，测试场景下可用）
    String::from_utf8(output.stdout).unwrap()
}

/// Write data to child process (cat) stdin via pipe and read its stdout output.
///
/// This demonstrates bidirectional pipe communication between parent and child processes.
///
/// # Underlying System Calls
/// - `Command::new("cat")` → `fork()` + `execve("cat")`
/// - `Stdio::piped()` (twice) → `pipe()` creates two pipes (stdin & stdout) + `dup2()` redirects them
/// - `ChildStdin::write_all()` → `write()` to the pipe's write end
/// - `drop(stdin)` → `close()` on the write end, sending EOF to child
/// - `ChildStdout::read_to_string()` → `read()` from the pipe's read end
///
/// # Ownership and Resource Management
/// Rust's ownership system ensures pipes are closed at the right time:
/// 1. The `ChildStdin` handle is owned by the parent; writing to it transfers data to the child.
/// 2. After writing, we explicitly `drop(stdin)` (or let it go out of scope) to close the write end.
/// 3. Closing the write end signals EOF to `cat`, causing it to exit after processing all input.
/// 4. The `ChildStdout` handle is then read to completion; dropping it closes the read end.
///
/// Without dropping `stdin`, the child would wait forever for more input (pipe never closes).
///
/// # Implementation Steps
/// 1. Create a `Command` for `"cat"` with `.stdin(Stdio::piped())` and `.stdout(Stdio::piped())`.
/// 2. `.spawn()` the command to obtain a `Child` with `stdin` and `stdout` handles.
/// 3. Write `input` bytes to the child's stdin (`child.stdin.take().unwrap().write_all(...)`).
/// 4. Drop the stdin handle (explicit `drop` or let it go out of scope) to close the pipe.
/// 5. Read the child's stdout (`child.stdout.take().unwrap().read_to_string(...)`).
/// 6. Wait for the child to exit with `.wait()` (or rely on drop‑wait).
pub fn pipe_through_cat(input: &str) -> String {
    // TODO: Create "cat" command, set stdin and stdout to piped
    // TODO: Spawn process
    // TODO: Write input to child process stdin
    // TODO: Drop stdin to close pipe (otherwise cat won't exit)
    // TODO: Read output from child process stdout
    let mut child = Command::new("cat")
        .stdin(Stdio::piped())  // 重定向stdin到管道（父进程写，子进程读）
        .stdout(Stdio::piped()) // 重定向stdout到管道（子进程写，父进程读）
        .spawn()                // 2. 启动子进程（不等待结束）
        .expect("Failed to spawn cat process");

    // 3. 向子进程stdin写入数据
    {
        let mut stdin = child.stdin.take().expect("Failed to get stdin");
        stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
        // 4. 显式drop关闭stdin管道，发送EOF给cat（否则cat会一直等待输入）
        drop(stdin);
    }

    // 5. 读取子进程stdout输出
    let mut stdout = child.stdout.take().expect("Failed to get stdout");
    let mut output = String::new();
    stdout.read_to_string(&mut output).expect("Failed to read from stdout");

    // 6. 等待子进程退出（可选，但更严谨）
    child.wait().expect("Failed to wait for child process");

    output
}

/// Get child process exit code.
/// Execute command `sh -c {command}` and return the exit code.
///
/// # Underlying System Calls
/// - `Command::new("sh")` → `fork()` + `execve("/bin/sh")`
/// - `.args(["-c", command])` passes the shell command line
/// - `.status()` → `waitpid()` (waits for child and retrieves exit status)
/// - `ExitStatus::code()` extracts the low‑byte exit code (0‑255)
///
/// # Implementation Steps
/// 1. Create a `Command` for `"sh"` with arguments `["-c", command]`.
/// 2. Call `.status()` to execute the shell and obtain an `ExitStatus`.
/// 3. Use `.code()` to get the exit code as `Option<i32>`.
/// 4. If the child terminated normally, return the exit code; otherwise return a default.
pub fn get_exit_code(command: &str) -> i32 {
    // TODO: Use Command::new("sh").args(["-c", command])
    // TODO: Execute and get status
    // TODO: Return exit code
    let status: ExitStatus = Command::new("sh")
        .args(["-c", command])
        .status()
        .expect("Failed to execute shell command");

    // 2. 获取退出码：正常终止返回code，否则返回-1（异常终止，如信号）
    status.code().unwrap_or(-1)
}

/// Execute the given shell command and return its stdout output as a `Result`.
///
/// This version properly propagates errors that may occur during process creation,
/// execution, or I/O (e.g., command not found, permission denied, broken pipe).
///
/// # Underlying System Calls
/// Same as `run_command`, but errors are captured from the OS and returned as `Err`.
///
/// # Error Handling
/// - `Command::new()` only constructs the builder; errors occur at `.output()`.
/// - `.output()` returns `Result<Output, std::io::Error>`.
/// - `String::from_utf8()` may fail if the child's output is not valid UTF‑8.
///   In that case we return an `io::Error` with kind `InvalidData`.
///
/// # Implementation Steps
/// 1. Create a `Command` with the given program and arguments.
/// 2. Set `.stdout(Stdio::piped())`.
/// 3. Call `.output()` and propagate any `io::Error`.
/// 4. Convert `stdout` to `String` with `String::from_utf8`; if that fails, map to an `io::Error`.
pub fn run_command_with_result(program: &str, args: &[&str]) -> io::Result<String> {
    // TODO: Use Command::new to create process
    // TODO: Set stdout to Stdio::piped()
    // TODO: Execute with .output() and handle Result
    // TODO: Convert stdout to String with from_utf8, mapping errors to io::Error
     let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .output()?; // 3. 传播io::Error

    // 4. 转换stdout为字符串，UTF8错误转为InvalidData类型的io::Error
    String::from_utf8(output.stdout)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Interact with `grep` via bidirectional pipes, filtering lines that contain a pattern.
///
/// This demonstrates complex parent‑child communication: the parent sends multiple
/// lines of input, the child (`grep`) filters them according to a pattern, and the
/// parent reads back only the matching lines.
///
/// # Underlying System Calls
/// - `Command::new("grep")` → `fork()` + `execve("grep")`
/// - Two pipes (stdin & stdout) as in `pipe_through_cat`
/// - Line‑by‑line writing and reading to simulate interactive filtering
///
/// # Implementation Steps
/// 1. Create a `Command` for `"grep"` with argument `pattern`, and both ends piped.
/// 2. `.spawn()` the command, obtaining `Child` with `stdin` and `stdout` handles.
/// 3. Write each line of `input` (separated by `'\n'`) to the child's stdin.
/// 4. Close the write end (drop stdin) to signal EOF.
/// 5. Read the child's stdout line by line, collecting matching lines.
/// 6. Wait for the child to exit (optional; `grep` exits after EOF).
/// 7. Return the concatenated matching lines as a single `String`.
///
pub fn pipe_through_grep(pattern: &str, input: &str) -> String {
    // TODO: Create "grep" command with pattern, set stdin and stdout to piped
    // TODO: Spawn process
    // TODO: Write input lines to child stdin
    // TODO: Drop stdin to close pipe
    // TODO: Read output from child stdout line by line
    // TODO: Collect and return matching lines
    let mut child = Command::new("grep")
        .arg(pattern)           // grep的匹配模式参数
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn grep process");

    // 2. 向grep的stdin写入多行输入
    {
        let mut stdin = child.stdin.take().expect("Failed to get grep stdin");
        // 3. 逐行写入（也可以直接写整个input，效果一致）
        stdin.write_all(input.as_bytes()).expect("Failed to write to grep stdin");
        // 4. 关闭stdin，发送EOF
        drop(stdin);
    }

    // 5. 读取grep的输出（按行读取）
    let stdout = child.stdout.take().expect("Failed to get grep stdout");
    let reader = BufReader::new(stdout);
    let mut output = String::new();

    for line in reader.lines() {
        let line = line.expect("Failed to read line from grep");
        output.push_str(&line);
        output.push('\n'); // 恢复换行符（lines()会去掉）
    }

    // 6. 等待子进程退出
    child.wait().expect("Failed to wait for grep process");

    output
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_run_echo() {
        let output = run_command("echo", &["hello"]);
        assert_eq!(output.trim(), "hello");
    }

    #[test]
    fn test_run_with_args() {
        let output = run_command("echo", &["-n", "no newline"]);
        assert_eq!(output, "no newline");
    }

    #[test]
    fn test_pipe_cat() {
        let output = pipe_through_cat("hello pipe!");
        assert_eq!(output, "hello pipe!");
    }

    #[test]
    fn test_pipe_multiline() {
        let input = "line1\nline2\nline3";
        assert_eq!(pipe_through_cat(input), input);
    }

    #[test]
    fn test_exit_code_success() {
        assert_eq!(get_exit_code("true"), 0);
    }

    #[test]
    fn test_exit_code_failure() {
        assert_eq!(get_exit_code("false"), 1);
    }

    #[test]
    fn test_run_command_with_result_success() {
        let result = run_command_with_result("echo", &["hello"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[test]
    fn test_run_command_with_result_nonexistent() {
        let result = run_command_with_result("nonexistent_command_xyz", &[]);
        // Should be an error because command not found
        assert!(result.is_err());
    }

    #[test]
    fn test_pipe_through_grep_basic() {
        let input = "apple\nbanana\ncherry\n";
        let output = pipe_through_grep("a", input);
        // grep outputs matching lines with newline
        assert_eq!(output, "apple\nbanana\n");
    }

    #[test]
    fn test_pipe_through_grep_no_match() {
        let input = "apple\nbanana\ncherry\n";
        let output = pipe_through_grep("z", input);
        // No lines match -> empty string
        assert_eq!(output, "");
    }

    #[test]
    fn test_pipe_through_grep_multiline() {
        let input = "first line\nsecond line\nthird line\n";
        let output = pipe_through_grep("second", input);
        assert_eq!(output, "second line\n");
    }
}
