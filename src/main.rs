use std::process::Stdio;
use std::sync::Arc;
use std::vec;

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let mut t0 = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("stdin manager start");

    let t0_stdout = t0.stdout.take().expect("mgr stdout");
    tokio::spawn(read_child_output(BufReader::new(t0_stdout), "mgr"));

    // create a mutex guard around the stdin manager's stdin stream
    let mut t0_stdin = t0.stdin.take().expect("stdin mgr stdin");
    let mutex = Arc::new(Mutex::new(&t0_stdin));

    tokio::spawn(async move {
        spawn_task("t1", "npm", vec!["run", "build-a"], &mutex);
    });

    tokio::spawn(async move {
        spawn_task("t2", "npm", vec!["run", "build-b"], &mutex);
    });

    // Read lines from stdin and send them to the stdin manager
    let mut in_reader = io::BufReader::new(io::stdin()).lines();
    while let Some(line) = in_reader.next_line().await.unwrap() {
        t0_stdin.write_all(line.as_bytes()).await.expect("");
        t0_stdin.write_all(b"\n").await.expect("");
    }
}

async fn spawn_task(
    name: &str,
    program: &str,
    args: Vec<&str>,
    mutex: &Mutex<&ChildStdin>,
) -> Child {
    let locked_stdin = mutex.lock().await;
    let xx = *locked_stdin;
    let yy = *xx;

    let mut cmd = Command::new(program);
    cmd.args(args.clone());
    cmd.stdout(Stdio::piped());
    cmd.stdin(yy);
    let haha = cmd.spawn().expect(format!("{} to start", name).as_str());

    let stdout = haha
        .stdout
        .take()
        .expect(format!("{} stdout", name).as_str());

    tokio::spawn(read_child_output(BufReader::new(stdout), "t1"));

    haha
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
