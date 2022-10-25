use std::{io, mem};
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::Ordering::Relaxed;

use tokio_uring::net::{TcpListener, TcpStream};

use crate::common::{consume_request, RESPONSE};
use crate::counter::COUNTER;

pub async fn new_http1_server(port: u16) -> io::Result<()> {
	let stream = TcpListener::bind(SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port))?;
	loop {
		let (client, _) = stream.accept().await?;
		handle_client(client).await?;
	}
}

async fn handle_client(client: TcpStream) -> io::Result<()> {
	let mut read_buf = vec![0u8; 16384];

	loop {
		let (res, buf) = client.read(read_buf).await;
		read_buf = buf;
		let len = res?;
		if len == 0 {
			break;
		}
		let mut content = &read_buf[..len];
		let mut remain = len;
		loop {
			let request_len = consume_request(content);
			if request_len == 0 {
				break;
			}
			remain -= request_len;
			content = &content[request_len..];
			send_response(&client, RESPONSE).await?;
		}
		let drain_from = len - remain;
		if drain_from < len {
			panic!("Errr? Big request is not supported at this time");
			//read_buf.drain(..=drain_from);
		}/* else {
			unsafe { read_buf.set_len(0); }
		}*/
	}
	Ok(())
}

async fn send_response(client: &TcpStream, buf: &[u8]) -> io::Result<()> {
	// construct vec from slice; and prevent it from drop!
	let buf = unsafe { Vec::from_raw_parts(buf.as_ptr() as *mut u8, buf.len(), buf.len()) };
	let (res, buf) = client.write(buf).await;
	mem::forget(buf);
	res?;
	COUNTER.fetch_add(1, Relaxed);
	Ok(())
}