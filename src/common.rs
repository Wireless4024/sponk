use bstr::ByteSlice;

pub static RESPONSE: &[u8] = br#"HTTP/1.1 200 OK
Server: Sponk
Content-Length: 13
Vary: Accept-Encoding
Connection: keep-alive
Keep-Alive: timeout=90
Content-Type: text/plain

Hello World!
"#;

/// Try to consume http request return size of http request
/// if request is invalid or ended will return 0
pub fn consume_request(buf: &[u8]) -> usize {
	let iter = buf.split(|&it| it == b'\n');
	let mut consume = 0;
	let mut content_len = 0;
	for line in iter {
		if line.is_empty() {
			break;
		}
		if line.starts_with(b"Content-Length") {
			content_len = line
				.rsplit(|&it| it == b':')
				.next()
				.and_then(|it| unsafe { it.as_bstr().trim_start().to_str_unchecked() }.parse::<usize>().ok())
				.unwrap_or_default();
		}
		consume += line.len() + 1;
	}
	let end = buf.len();
	if end < consume + content_len {
		0
	} else {
		let res = consume + content_len;
		res + usize::from(end != res)
	}
}


#[cfg(test)]
mod test {
	use crate::common::consume_request;

	#[test]
	fn test_consume() {
		let request = br#"GET /hello.txt HTTP/1.1
User-Agent: curl/7.64.1
Host: www.example.com
Accept-Language: en, mi

GET /hello.txt HTTP/1.1
User-Agent: curl/7.64.1
Host: www.example.com
Accept-Language: en, mi
Content-Length: 6

Hello
"#;
		assert_eq!(consume_request(request), 95);
		assert_eq!(consume_request(&request[95..]), 119);
	}
}