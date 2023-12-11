use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {
    // Spawn a child process
    let mut child = Command::new("bash")
        .arg("src/request-input.sh")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start child process");

    // Get a handle to the child process's stdin
    let mut child_stdin = child.stdin.take().expect("Failed to get child stdin");

    // Get a handle to the child process's stdout
    let child_stdout = child.stdout.take().expect("Failed to get child stdout");

    // Use Tokio to read lines from the child process's stdout asynchronously
    let reader = BufReader::new(child_stdout);
    tokio::spawn(read_child_output(reader));

    // Use Tokio to read lines from stdin and send them to the child process
    let stdin = io::stdin();
    let mut lines = io::BufReader::new(stdin).lines();

    while let Some(line) = lines.next_line().await.unwrap() {
        println!("app-a:build: requesting input");
        print!("app-a:build: {}", line.trim());
        // Send the input line to the child process
        child_stdin
            .write_all(line.as_bytes())
            .await
            .expect("Failed to write to child stdin");
        child_stdin
            .write_all(b"\n")
            .await
            .expect("Failed to write newline to child stdin");
    }

    // Keep the parent process alive for some time
    time::sleep(Duration::from_secs(5)).await;

    // Send an interrupt signal to the child process
    child.kill().await.expect("Failed to kill child process");

    // Wait for the child process to exit
    let status = child
        .wait()
        .await
        .expect("Failed to wait for child process");
    println!("Child process exited with: {:?}", status);
}

async fn read_child_output(mut reader: BufReader<tokio::process::ChildStdout>) {
    let mut line = String::new();
    while let Ok(n) = reader.read_line(&mut line).await {
        if n == 0 {
            // End of file reached
            break;
        }
        print!("{}", line);
        line.clear();
    }
}
