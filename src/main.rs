use std::thread::{JoinHandle, spawn};

use crate::h1::new_http1_server;

mod h1;

fn main() {
	new_server(8000).join().expect("TODO: panic message");
}

fn new_server(port: u16) -> JoinHandle<()> {
	spawn(move || {
		tokio_uring::start(async {
			new_http1_server(port).await.ok();
		})
	})
}