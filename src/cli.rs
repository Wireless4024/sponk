use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct SponkCli {
	#[arg(short = 'H', default_value = "1")]
	pub http_version: usize,
	#[arg(short = 'p', default_value = "8080")]
	pub port: u16,
	#[arg(short = 'c', default_value = "16")]
	pub concurrency: usize,
	#[arg(short = 't', default_value = "0")]
	pub threads: usize,
}