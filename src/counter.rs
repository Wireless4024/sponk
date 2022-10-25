use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{JoinHandle, sleep, spawn};
use std::time::Duration;

pub static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn start_monitor() -> JoinHandle<()> {
	println!("starting monitor thread");
	spawn(|| {
		loop {
			sleep(Duration::from_secs(1));
			let last = COUNTER.swap(0, Ordering::Relaxed);
			if last > 0 {
				let level = (last as f64).log10().ceil() as usize / 2;
				let level = "!".repeat(level);
				println!("OMG{level} there are {last} requests in last second{level}");
			}
		}
	})
}