#![feature(test)]

extern crate test;
extern crate rand;
extern crate pnet;
extern crate byteorder;

use test::Bencher;

use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ethernet::EthernetPacket;

use rand::{Rand,Rng};
use std::io::{Seek,Cursor};
use std::net::Ipv4Addr;
use pnet::util::MacAddr;
use byteorder::{BigEndian, ReadBytesExt};

const PACKET: [u8; 60] = [
 		0x45, 0x0, 0x0, 0x3c, 0x30, 0x66, 0x40, 0x0, 0x40, 0x06, 0xc,
		0x54, 0x7f, 0x0, 0x0, 0x01, 0x7f, 0x0, 0x0, 0x02, 0xdc, 0x52,
		0x0, 0x50, 0x04, 0x54, 0x76, 0x5d, 0x0, 0x0, 0x0, 0x0, 0xa0,
		0x02, 0xaa, 0xaa, 0xfe, 0x30, 0x0, 0x0, 0x02, 0x04, 0xff, 0xd7,
		0x04, 0x02, 0x08, 0x0a, 0x05, 0x01, 0x45, 0x3b, 0x0, 0x0, 0x0,
		0x0, 0x01, 0x03, 0x03, 0x07 ];


const ETHER_PACKET: [u8; 14] = [
		0xbc, 0x5f, 0xf4, 0x36, 0x5a, 0xbe, 0x68, 0x05, 0xca, 0x21, 0x58, 0x86, 0x08, 0x00
];

fn bytes_get_src_dst(bytes: &[u8]) -> (Ipv4Addr, Ipv4Addr) {
	let mut buf: [u8; 4] = [0; 4];
	(&mut buf).copy_from_slice(&bytes[12..16]);
	let src = Ipv4Addr::from(buf);
	let mut buf: [u8; 4] = [0; 4];
	(&mut buf).copy_from_slice(&bytes[16..20]);
	let dst = Ipv4Addr::from(buf);
	(src, dst)
}

fn byteorder_get_src_dst(bytes: &[u8]) -> std::io::Result<(Ipv4Addr, Ipv4Addr)> {
	let mut rdr = Cursor::new(bytes);
	try!(rdr.seek(std::io::SeekFrom::Start(12)));
	let src = try!(rdr.read_u32::<BigEndian>());
	let dst = try!(rdr.read_u32::<BigEndian>());
	Ok((Ipv4Addr::from(src), Ipv4Addr::from(dst)))
}

fn pnet_get_src_dst(bytes: &[u8]) -> Option<(Ipv4Addr, Ipv4Addr)> {
	if let Some(ipv4) = Ipv4Packet::new(bytes) {
		let src = ipv4.get_source();
		let dst = ipv4.get_destination();
		return Some((src, dst));
	}
	None
}

#[test]
fn get_src_dst() {
	let src = Ipv4Addr::new(127, 0, 0, 1);
	let dst = Ipv4Addr::new(127, 0, 0, 2);

	if let Some((pnet_src, pnet_dst)) = pnet_get_src_dst(&PACKET) {
		assert_eq!(pnet_src, src);
		assert_eq!(pnet_dst, dst);
	}
	if let Ok((bo_src, bo_dst)) = byteorder_get_src_dst(&PACKET) {
		assert_eq!(bo_src, src);
		assert_eq!(bo_dst, dst);
	}
	let (by_src, by_dst) = bytes_get_src_dst(&PACKET);
	assert_eq!(by_src, src);
	assert_eq!(by_dst, dst);

}

#[bench]
fn get_src_dst_pnet(b: &mut Bencher) {
	let mut rng = rand::thread_rng();
	b.iter(|| {
		let mut bytes: [u8; 32] = rng.gen();
		test::black_box(pnet_get_src_dst(&bytes));
	})
}

#[bench]
fn get_src_dst_byteorder(b: &mut Bencher) {
	let mut rng = rand::thread_rng();
	b.iter(|| {
		let mut bytes: [u8; 32] = rng.gen();
		test::black_box(byteorder_get_src_dst(&bytes));
	})
}

#[bench]
fn get_src_dst_bytes(b: &mut Bencher) {
	let mut rng = rand::thread_rng();
	b.iter(|| {
		let mut bytes: [u8; 32] = rng.gen();
		test::black_box(bytes_get_src_dst(&bytes));
	})
}

#[bench]
fn compare_mac_addr_libpnet(b: &mut Bencher) {
	let mut rng = rand::thread_rng();

	let tuple: (u8, u8, u8, u8, u8, u8) = rng.gen();
	let mac = MacAddr::new(tuple.0, tuple.1, tuple.2, tuple.3, tuple.4, tuple.5);
	b.iter(|| {
		let rand_tuple: (u8, u8, u8, u8, u8, u8) = rng.gen();
		let rand_mac = MacAddr::new(tuple.0, tuple.1, tuple.2, tuple.3, tuple.4, tuple.5);
		test::black_box(mac == rand_mac);
	})
}

struct MacAddr2([u8; 6]);

impl MacAddr2 {
	fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> MacAddr2 {
		MacAddr2([a, b, c, d, e, f])
	}
}

impl PartialEq for MacAddr2 {
	fn eq(&self, other: &MacAddr2) -> bool {
		self.0 == other.0
	}
}

impl From<[u8; 6]> for MacAddr2 {
	fn from(bytes: [u8; 6]) -> Self {
		MacAddr2(bytes)
	}
}

#[bench]
fn compare_mac_addr_arr(b: &mut Bencher) {
	let mut rng = rand::thread_rng();

	let tuple: (u8, u8, u8, u8, u8, u8) = rng.gen();
	let mac = MacAddr2::new(tuple.0, tuple.1, tuple.2, tuple.3, tuple.4, tuple.5);
	b.iter(|| {
		let rand_tuple: (u8, u8, u8, u8, u8, u8) = rng.gen();
		let rand_mac = MacAddr2::new(tuple.0, tuple.1, tuple.2, tuple.3, tuple.4, tuple.5);
		test::black_box(mac == rand_mac);
	})
}

struct MacAddr3([u8; 6]);

impl MacAddr3 {
	fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> MacAddr3 {
		MacAddr3([a, b, c, d, e, f])
	}
}

impl PartialEq for MacAddr3 {
	fn eq(&self, other: &MacAddr3) -> bool {
		use std::mem;

		let a = self.0.as_ptr();
		let b = other.0.as_ptr();

		let a: *const u8  = unsafe { mem::transmute(a) };
		let b: *const u8  = unsafe { mem::transmute(b) };

		let a1: *const u32 = unsafe { mem::transmute(a) };
		let a2: *const u32 = unsafe { mem::transmute(a.offset(2)) };

		let b1: *const u32 = unsafe { mem::transmute(b) };
		let b2: *const u32 = unsafe { mem::transmute(b.offset(2)) };

		unsafe { *a1 == *b1 && *a2 == *b2 }
	}
}

#[bench]
fn compare_mac_addr_transmute(b: &mut Bencher) {
	let mut rng = rand::thread_rng();

	let tuple: (u8, u8, u8, u8, u8, u8) = rng.gen();
	let mac = MacAddr3::new(tuple.0, tuple.1, tuple.2, tuple.3, tuple.4, tuple.5);
	b.iter(|| {
		let rand_tuple: (u8, u8, u8, u8, u8, u8) = rng.gen();
		let rand_mac = MacAddr3::new(tuple.0, tuple.1, tuple.2, tuple.3, tuple.4, tuple.5);
		test::black_box(mac == rand_mac);
	})
}


fn pnet_get_dest_mac(bytes: &[u8]) -> Option<MacAddr> {
	if let Some(eth) = EthernetPacket::new(bytes) {
		return Some(eth.get_destination());
	}
	None
}

#[bench]
fn compare_mac_addr_from_packet_pnet(b: &mut Bencher) {
	let mut rng = rand::thread_rng();

	let tuple: (u8, u8, u8, u8, u8, u8) = rng.gen();
	let my_mac = MacAddr::new(tuple.0, tuple.1, tuple.2, tuple.3, tuple.4, tuple.5);

	b.iter(|| {
		let mut bytes: [u8; 14] = rng.gen();
		let mac = pnet_get_dest_mac(&bytes).unwrap();
		test::black_box(mac == my_mac);
	})
}

struct MacAddrBuf<'a>(&'a [u8]);

impl<'a> PartialEq for MacAddrBuf<'a> {
	fn eq(&self, other: &MacAddrBuf<'a>) -> bool {
		self.0 == other.0
	}	
}

fn slice_get_dest_mac(bytes: &[u8]) -> Option<MacAddrBuf> {
	if bytes.len() >= 12 {
		return Some(MacAddrBuf(&bytes[6..12]))
	}
	None
}

#[bench]
fn compare_mac_addr_from_packet_slice(b: &mut Bencher) {
	let mut rng = rand::thread_rng();

	let tuple: [u8; 6] = rng.gen();
	let my_mac = MacAddrBuf(&tuple);

	b.iter(|| {
		let mut bytes: [u8; 14] = rng.gen();
		let mac = slice_get_dest_mac(&bytes).unwrap();
		test::black_box(mac == my_mac);
	})
}


fn bytes_get_dest_mac(bytes: &[u8]) -> Option<MacAddr2> {
	if bytes.len() >= 12 {
		let mut buf: [u8; 6] = unsafe { std::mem::uninitialized() };
		(&mut buf).copy_from_slice(&bytes[6..12]);
		return Some(MacAddr2::from(buf));
	}
	None
}

#[bench]
fn compare_mac_addr_from_packet_bytes(b: &mut Bencher) {
	let mut rng = rand::thread_rng();

	let tuple: (u8, u8, u8, u8, u8, u8) = rng.gen();
	let my_mac = MacAddr2::new(tuple.0, tuple.1, tuple.2, tuple.3, tuple.4, tuple.5);

	b.iter(|| {
		let mut bytes: [u8; 14] = rng.gen();
		let mac = bytes_get_dest_mac(&bytes).unwrap();
		test::black_box(mac == my_mac);
	})
}
