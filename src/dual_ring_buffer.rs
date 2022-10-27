use std::{io, mem};
use std::cmp::min;
use std::ops::Deref;

use bstr::ByteVec;
use tokio_uring::net::TcpStream;

pub struct DualRingBuffer {
	buffer: Vec<u8>,
	buffer_len: usize,
	buffer_ptr: usize,
	temp: Vec<u8>,
	temp_len: usize,
	temp_ptr: usize,
	capacity: usize,
}

impl DualRingBuffer {
	pub fn new(capacity: usize) -> Self {
		Self {
			buffer: vec![0; capacity],
			buffer_len: 0,
			buffer_ptr: 0,
			temp: vec![0; capacity],
			temp_len: 0,
			temp_ptr: 0,
			capacity: capacity,
		}
	}

	pub fn append(&mut self, data: &[u8]) {
		let mut buf = self.prep_read();
		buf.replace_range(..data.len(), data);
		self.push_buf(buf, data.len());
	}

	fn push_buf(&mut self, data: Vec<u8>, len: usize) -> bool {
		self.buffer = data;
		self.buffer_ptr = 0;
		self.buffer_len = len;
		self.buffer_len != 0
	}

	pub async fn read_tcp(&mut self, client: &TcpStream) -> io::Result<bool> {
		let buffer = self.prep_read();
		let (res, buf) = client.read(buffer).await;
		Ok(self.push_buf(buf, res?))
	}

	/// Copy all remaining data from buffer to temp
	#[inline]
	fn prep_read(&mut self) -> Vec<u8> {
		let len = self.buffer_len;
		let cap = self.capacity;

		if len > 0 {
			let tmp_len = self.temp_len;
			if tmp_len == 0 {
				self.temp_len = mem::replace(&mut self.buffer_len, 0);
				if self.temp.capacity() > cap {
					self.temp_ptr = mem::replace(&mut self.buffer_ptr, 0);
					mem::swap(&mut self.buffer, &mut self.temp);
				} else {
					// resize temp if its oversize
					let mut vec = Vec::with_capacity(cap);
					let temp_ptr = mem::replace(&mut self.buffer_ptr, 0);
					vec.extend(&self.buffer[temp_ptr..temp_ptr + len]);
					unsafe { vec.set_len(cap) };
					self.temp_ptr = 0;
					self.temp = vec;
				}
			} else {
				let ptr = self.buffer_ptr;
				let end = ptr + len;

				// drain in front of temp and move ptr to 0
				// remaining data should be small drain should not be a problem
				let tmp_off = mem::replace(&mut self.temp_ptr, 0);
				unsafe { drain_head(&mut self.temp, tmp_off, tmp_len) };

				// copy remaining data from buffer to temp
				let final_len = tmp_len + len;
				let tmp_cap = self.temp.capacity();
				// u8 should be fine
				#[allow(clippy::uninit_vec)]
				if final_len >= tmp_cap {
					self.temp.reserve(self.temp.capacity());
					unsafe { self.temp.set_len(self.temp.capacity()); }
				}
				self.temp.replace_range(tmp_len..final_len, &self.buffer[ptr..end]);
				self.temp_len = tmp_len + len;
			}
		}
		// should be empty buffer
		let mut buffer = mem::replace(&mut self.buffer, vec![0; cap]);
		if buffer.capacity() > cap {
			buffer = vec![0; cap];
		}
		buffer
	}

	pub fn read(&mut self, mut target: &mut [u8]) -> usize {
		let tlen = self.temp_len;
		let mut read_len = 0;
		if tlen > 0 {
			// copy from temp first
			let need = min(target.len(), tlen);
			if need == tlen {
				// take all
				let temp_ptr = mem::replace(&mut self.temp_ptr, 0);
				let temp_len = mem::replace(&mut self.temp_len, 0);
				target[..need].copy_from_slice(&self.temp[temp_ptr..temp_ptr + temp_len]);
			} else {
				// take part of it
				let temp_ptr = self.temp_ptr;
				let temp_len = self.temp_len;
				target[..need].copy_from_slice(&self.temp[temp_ptr..temp_ptr + temp_len]);
				self.temp_ptr += need;
				self.temp_len -= need;
			}
			read_len += need;
		}
		let blen = self.buffer_len;
		let need = min(target.len(), blen);
		// take part of it
		let buff_ptr = self.buffer_ptr;
		let buff_len = self.buffer_len;
		target[read_len..read_len + need].copy_from_slice(&self.buffer[buff_ptr..buff_ptr + buff_len]);
		self.buffer_ptr += need;
		self.buffer_len -= need;
		if self.buffer_len == 0 {
			self.buffer_ptr = 0;
		}
		read_len += need;
		read_len
	}

	pub fn try_swap(&mut self, mut data: Vec<u8>) -> Buffer {
		if data.capacity() != self.capacity {
			data = vec![0; self.capacity];
		}
		if self.temp_len == 0 {
			let inner = mem::replace(&mut self.buffer, data);
			Buffer {
				inner,
				ptr: mem::replace(&mut self.buffer_ptr, 0),
				len: mem::replace(&mut self.buffer_len, 0),
			}
		} else {
			let inner = mem::replace(&mut self.temp, data);
			Buffer {
				inner,
				ptr: mem::replace(&mut self.temp_ptr, 0),
				len: mem::replace(&mut self.temp_len, 0),
			}
		}
	}

	#[inline]
	pub fn should_fill(&self) -> bool {
		self.temp_len == 0
	}
}

#[derive(Debug)]
pub struct Buffer {
	pub inner: Vec<u8>,
	pub ptr: usize,
	pub len: usize,
}

impl Deref for Buffer {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.inner[self.ptr..self.ptr + self.len]
	}
}

#[inline]
unsafe fn drain_head(it: &mut Vec<u8>, off: usize, len: usize) {
	let ptr = it.as_mut_ptr();
	let data_ptr = ptr.add(off);
	ptr.copy_from(data_ptr, len);
}

#[cfg(test)]
mod tests {
	use bstr::ByteSlice;

	use crate::dual_ring_buffer::{drain_head, DualRingBuffer};

	#[test]
	fn test_drain() {
		let mut arr = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
		unsafe { drain_head(&mut arr, 5, 4); }
		assert_eq!(arr, vec![5, 6, 7, 8]);
	}

	#[test]
	fn test_buffer() {
		let mut drb = DualRingBuffer::new(32);
		drb.append(b"Hello ");
		drb.append(b"World");
		let mut data = vec![0; 16];
		let len = drb.read(&mut data);
		unsafe {
			data.set_len(len);
		}
		assert_eq!(data.as_bstr(), "Hello World");
	}

	#[test]
	fn test_swap() {
		let cap = 3;
		let mut drb = DualRingBuffer::new(cap);
		drb.append(b"foo");
		drb.append(b"bar");
		drb.append(b"baz");
		drb.append(b"qux");
		let res = drb.try_swap(Vec::new());
		//println!("{:?}", res);
		assert!(res.inner.capacity() > 9);
		assert_eq!(res.len, 9);
		let res = drb.try_swap(res.inner);
		//println!("{:?}", res);
		assert_eq!(res.inner.capacity(), cap);
		assert_eq!(res.len, cap);
		drb.append(b"foo");
		let res = drb.try_swap(res.inner);
		//println!("{:?}", res);
		assert_eq!(res.inner.capacity(), cap);
		assert_eq!(res.len, cap);
		drb.append(b"bar");
		let res = drb.try_swap(res.inner);
		//println!("{:?}", res);
		assert_eq!(res.inner.capacity(), cap);
		assert_eq!(res.len, cap);
	}
}