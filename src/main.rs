use std::env;
use std::os::unix::io::{FromRawFd, RawFd};
use actix_web::{web, App, HttpServer, Responder};
use std::os::unix::net::UnixListener;

async fn index() -> impl Responder {
    let pid = std::process::id();
    format!("Hello from actix-web with systemd socket! Current process ID: {}", pid)
}

#[tokio::main]
async fn main() {
    // Environment variables set by systemd
    let listen_fds = env::var("LISTEN_FDS").expect("LISTEN_FDS not set").parse::<i32>().expect("Invalid LISTEN_FDS");
    let listen_pid = env::var("LISTEN_PID").expect("LISTEN_PID not set").parse::<i32>().expect("Invalid LISTEN_PID");

    // Ensure that this process is the one systemd intended to start
    if listen_pid != std::process::id() as i32 {
        panic!("LISTEN_PID does not match the current process");
    }

    // File descriptor provided by systemd (starts at 3)
    let fd: RawFd = 3;
    
    // Only continue if systemd passed a file descriptor
    if listen_fds > 0 {
        let listener = unsafe { UnixListener::from_raw_fd(fd) };
        
        HttpServer::new(|| {
            App::new().route("/", web::get().to(index))
        })
        .listen_uds(listener)
        .expect("Failed to bind to systemd provided socket")
        .run()
        .await
        .expect("Failed to run server");
    } else {
        panic!("No sockets passed by systemd");
    }
}
