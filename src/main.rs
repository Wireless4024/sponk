use std::process::exit;
use std::thread::{JoinHandle, spawn};

use clap::Parser;
use futures::stream::FuturesUnordered;
use futures::StreamExt;

use crate::cli::SponkCli;
use crate::counter::start_monitor;
use crate::h1_uring::new_http1_server;

mod h1_uring;
mod cli;
mod counter;
mod common;

pub type Server = fn(u16, usize) -> JoinHandle<()>;

fn main() {
	#[cfg(not(target_os = "linux"))]
	compile_error!("only linux is supported");
	let args = SponkCli::parse();
	let start: Server = match args.http_version {
		1 => { new_h1_server }
		v => {
			eprintln!("Http version {v} is not supported");
			exit(1);
		}
	};
	println!("starting server");
	let port = args.port;
	let n_threads = if args.threads == 0 {
		num_cpus::get()
	} else {
		args.threads
	};
	let mut threads = Vec::with_capacity(n_threads + 1);
	for i in 1..=n_threads {
		println!("starting thread {i}");
		let t = start(port, args.concurrency);
		threads.push(t);
	}
	threads.push(start_monitor());
	println!("server started! on port {}", port);
	println!("ctrl+c to stop");
	for x in threads {
		x.join().expect("Failed to join threads");
	}
}

fn new_h1_server(port: u16, concurrency: usize) -> JoinHandle<()> {
	spawn(move || {
		tokio_uring::start(async {
			let futures = FuturesUnordered::new();
			for _ in 0..concurrency {
				futures.push(new_http1_server(port));
			}
			let _: Vec<_> = futures.collect().await;
		});
	})
}