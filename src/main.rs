use std::os::unix::io::{FromRawFd, RawFd};
use actix_web::{web, App, HttpServer, Responder};
use std::os::unix::net::UnixListener;

async fn index() -> impl Responder {
    let pid = std::process::id();
    format!("Hello from actix-web with systemd socket! Current process ID: {}", pid)
}

#[tokio::main]
async fn main() {
    let listen_fds: RawFd = 3;
    let listener = unsafe { UnixListener::from_raw_fd(listen_fds) };

    HttpServer::new(|| {
        App::new().route("/", web::get().to(index))
    })
    .listen_uds(listener)
    .expect("Failed to bind to systemd provided socket")
    .run()
    .await
    .expect("Failed to run server");
}
