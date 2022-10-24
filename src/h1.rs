use std::io;
use std::net::{IpAddr, SocketAddr};
use tokio_uring::net::TcpListener;

pub async fn new_http1_server(port: u16) -> io::Result<()> {
	let stream = TcpListener::bind(SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port))?;
	let (client, _) = stream.accept().await?;
	loop {

	}
	Ok(())
}