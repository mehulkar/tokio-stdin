use std::vec;

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

#[tokio::main]
async fn main() {
    let mut stdin_mgr = Command::new("cat")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("stdin manager start");

    let mut task_one = Command::new("npm")
        .args(vec!["run", "start"])
        .stdout(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .spawn()
        .expect("t1 start");

    let mut task_two = Command::new("npm")
        .args(vec!["run", "dev"])
        .stdout(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .spawn()
        .expect("t2 start");

    let mut stdin_mgr_stdin = stdin_mgr.stdin.take().expect("stdin mgr stdin");

    // Get a handle to each process's stdout and stdin
    // Read and print stdout
    let t1_stdout = task_one.stdout.take().expect("t1 stdout");
    let t2_stdout = task_two.stdout.take().expect("t2 stdout");
    let mgr_stdout = stdin_mgr.stdout.take().expect("mgr stdout");
    tokio::spawn(read_child_output(BufReader::new(t1_stdout), "t1"));
    tokio::spawn(read_child_output(BufReader::new(t2_stdout), "t2"));
    tokio::spawn(read_child_output(BufReader::new(mgr_stdout), "mgr"));

    // Use Tokio to read lines from stdin and send them to the stdin manager
    let mut in_reader = io::BufReader::new(io::stdin()).lines();

    while let Some(line) = in_reader.next_line().await.unwrap() {
        stdin_mgr_stdin.write_all(line.as_bytes()).await.expect("");
        stdin_mgr_stdin.write_all(b"\n").await.expect("");
    }
}

async fn read_child_output(mut reader: BufReader<tokio::process::ChildStdout>, prefix: &str) {
    let mut line = String::new();
    while let Ok(n) = reader.read_line(&mut line).await {
        // End of file reached
        if n == 0 {
            break;
        }
        print!("{}: {}", prefix, line);
        line.clear();
    }
}
