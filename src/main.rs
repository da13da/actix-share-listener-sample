use nix::unistd::{fork, ForkResult};
use actix_web::{web, App, HttpServer, Responder};
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::process::{exit};
use tokio::runtime::Runtime;

async fn index() -> impl Responder {
    let pid = std::process::id();
    format!("Hello from actix-web with UDS! Current process ID: {}", pid)
}

fn launch_process(fd: RawFd, process_name: &str) {
    println!("Starting process {} with fd {}", process_name, fd);

    let listener = unsafe { UnixListener::from_raw_fd(fd) };

    println!("Process {}: listener created from fd", process_name);

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        HttpServer::new(move || {
            App::new().route("/", web::get().to(index))
        })
        .listen_uds(listener)
        .expect("error binding to socket")
        .run()
        .await
        .expect("Server failed to run");
    });

    println!("Process {} is running the server", process_name);
    exit(0);
}

fn fork_and_launch(process_name: &str, listener_fd: RawFd) {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            println!("Parent process Forked child process {} with PID {}", process_name, child);
        }
        Ok(ForkResult::Child) => {
            let pid = std::process::id();
            println!("Child process {} (PID: {}) is starting with fd {}", process_name, pid, listener_fd);
            launch_process(listener_fd, process_name);
        }
        Err(e) => {
            eprintln!("Failed to fork process {}: {:?}", process_name, e);
        }
    }
}

fn main() {
    let uds_path = "/tmp/share-fd.sock";
    if Path::new(uds_path).exists() {
        std::fs::remove_file(uds_path).unwrap();
    }

    let listener = UnixListener::bind(uds_path).unwrap();
    let listener_fd = listener.as_raw_fd();

    println!("Parent process starting processes child1 and child2");
    fork_and_launch("child1", listener_fd);
    fork_and_launch("child2", listener_fd);

    println!("Parent process finished");
}
