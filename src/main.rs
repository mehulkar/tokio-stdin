use std::vec;

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let mut task_zero = Command::new("cat")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("stdin manager start");

    let t0_stdout = task_zero.stdout.take().expect("mgr stdout");
    tokio::spawn(read_child_output(BufReader::new(t0_stdout), "mgr"));
    let mutex = Mutex::new(0);
    let t1 = spawn_task("t1", "npm", vec!["run", "build-a"], &mutex);
    println!("spawning task 2!");
    let t2 = spawn_task("t2", "npm", vec!["run", "build-b"], &mutex);

    // Use Tokio to read lines from stdin and send them to the stdin manager
    let mut t0_stdin = task_zero.stdin.take().expect("stdin mgr stdin");

    let mut in_reader = io::BufReader::new(io::stdin()).lines();
    while let Some(line) = in_reader.next_line().await.unwrap() {
        t0_stdin.write_all(line.as_bytes()).await.expect("");
        t0_stdin.write_all(b"\n").await.expect("");
    }

    // TODO: Wait for both asynchronously instead of blocking like this
    println!("done");
}

fn spawn_task(name: &str, program: &str, args: Vec<&str>, mutex: &Mutex<i32>) -> Child {
    println!("about to lock the mutex: {}", name);
    let _guard = mutex.lock();

    println!("spawn_task: {} has the lock!", name);

    let mut cmd = Command::new(program)
        .args(args.clone())
        .stdout(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .spawn()
        .expect(format!("{} {:?} to start", program, args.clone()).as_str());

    let stdout = cmd
        .stdout
        .take()
        .expect(format!("{} stdout", name).as_str());

    println!("reading child ouptut {}!", name);

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
