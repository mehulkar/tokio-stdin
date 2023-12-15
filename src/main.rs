use std::process::Stdio;
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
    let mut t0_stdin = t0.stdin.take().expect("stdin mgr stdin");

    tokio::spawn(read_child_output(BufReader::new(t0_stdout), "mgr"));

    // create a mutex guard around the stdin manager's stdin stream
    let mutex = Mutex::new(&t0_stdin);
    let mut t1 = spawn_task("t1", "npm", vec!["run", "build-a"], &mutex);
    let mut t2 = spawn_task("t2", "npm", vec!["run", "build-b"], &mutex);

    // Read lines from stdin and send them to the stdin manager
    let mut in_reader = io::BufReader::new(io::stdin()).lines();
    while let Some(line) = in_reader.next_line().await.unwrap() {
        t0_stdin.write_all(line.as_bytes()).await.expect("");
        t0_stdin.write_all(b"\n").await.expect("");
    }

    t0.wait().await.unwrap();
    t1.await.wait().await.unwrap();
    t2.await.wait().await.unwrap();
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

    println!("spawn_task: {} has the lock!", name);

    let mut cmd = Command::new(program)
        .args(args.clone())
        .stdout(Stdio::piped())
        .stdin(yy)
        .spawn()
        .expect(format!("{} to start", name).as_str());

    let stdout = cmd
        .stdout
        .take()
        .expect(format!("{} stdout", name).as_str());

    tokio::spawn(read_child_output(BufReader::new(stdout), "t1"));

    cmd
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
