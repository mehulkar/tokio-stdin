use std::sync::{Arc, Mutex};
use std::vec;

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

pub struct Foo {
    pub lock: Arc<Mutex<i32>>,
}

impl Foo {
    pub fn new() -> Foo {
        Foo {
            lock: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn start(&self) {
        let mut stdin_mgr = Command::new("cat")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("stdin manager start");

        let mut task_one = self.spawn_task("npm", vec!["run", "start"]);
        let mut task_two = self.spawn_task("npm", vec!["run", "dev"]);

        // Read and print stdout
        let t1_stdout = task_one.stdout.take().expect("t1 stdout");
        let t2_stdout = task_two.stdout.take().expect("t2 stdout");
        let mgr_stdout = stdin_mgr.stdout.take().expect("mgr stdout");
        tokio::spawn(read_child_output(BufReader::new(t1_stdout), "t1"));
        tokio::spawn(read_child_output(BufReader::new(t2_stdout), "t2"));
        tokio::spawn(read_child_output(BufReader::new(mgr_stdout), "mgr"));

        // Use Tokio to read lines from stdin and send them to the stdin manager
        let mut stdin_mgr_stdin = stdin_mgr.stdin.take().expect("stdin mgr stdin");
        let mut in_reader = io::BufReader::new(io::stdin()).lines();
        while let Some(line) = in_reader.next_line().await.unwrap() {
            stdin_mgr_stdin.write_all(line.as_bytes()).await.expect("");
            stdin_mgr_stdin.write_all(b"\n").await.expect("");
        }
    }

    fn spawn_task(&self, program: &str, args: Vec<&str>) -> Child {
        Command::new(program)
            .args(args.clone())
            .stdout(std::process::Stdio::piped())
            .stdin(std::process::Stdio::null())
            .spawn()
            .expect(format!("{} {:?} to start", program, args.clone()).as_str())
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

#[tokio::main]
async fn main() {
    let foo = Foo::new();
    foo.start().await;
}
