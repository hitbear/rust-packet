//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                    Version 2, December 2004
//
// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// Everyone is permitted to copy and distribute verbatim or modified
// copies of this license document, and changing it is allowed as long
// as the name is changed.
//
//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
//
//  0. You just DO WHAT THE FUCK YOU WANT TO.

use std::fmt;
use byteorder::{ReadBytesExt, BigEndian};

use error::*;
use size;
use packet::Packet as P;
use ip;
use udp::checksum;

pub struct Packet<B> {
	buffer: B,
}

sized!(Packet,
	header {
		min:  8,
		max:  8,
		size: 8,
	}

	payload {
		min:  0,
		max:  u16::max_value() as usize - 8,
		size: p => p.length() as usize - 8,
	});

impl<B: AsRef<[u8]>> fmt::Debug for Packet<B> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("udp::Packet")
			.field("source", &self.source())
			.field("destination", &self.destination())
			.field("checksum", &self.length())
			.field("checksum", &self.checksum())
			.field("payload", &self.payload())
			.finish()
	}
}

impl<B: AsRef<[u8]>> Packet<B> {
	pub fn new(buffer: B) -> Result<Packet<B>> {
		let packet = Packet {
			buffer: buffer,
		};

		if packet.buffer.as_ref().len() < <Self as size::header::Min>::min() {
			return Err(ErrorKind::InvalidPacket.into());
		}

		Ok(packet)
	}
}

impl<B: AsRef<[u8]>> Packet<B> {
	pub fn source(&self) -> u16 {
		(&self.buffer.as_ref()[0 ..]).read_u16::<BigEndian>().unwrap()
	}

	pub fn destination(&self) -> u16 {
		(&self.buffer.as_ref()[2 ..]).read_u16::<BigEndian>().unwrap()
	}

	pub fn length(&self) -> u16 {
		(&self.buffer.as_ref()[4 ..]).read_u16::<BigEndian>().unwrap()
	}

	pub fn checksum(&self) -> u16 {
		(&self.buffer.as_ref()[6 ..]).read_u16::<BigEndian>().unwrap()
	}

	pub fn is_valid<I: AsRef<[u8]>>(&self, ip: &ip::Packet<I>) -> bool {
		checksum(ip, self.buffer.as_ref()) == self.checksum()
	}
}

impl<B: AsRef<[u8]>> P for Packet<B> {
	fn header(&self) -> &[u8] {
		&self.buffer.as_ref()[.. 8]
	}

	fn payload(&self) -> &[u8] {
		&self.buffer.as_ref()[8 ..]
	}
}

#[cfg(test)]
mod test {
	use packet::Packet;
	use ip;
	use udp;

	#[test]
	fn values() {
		let raw = [0x45u8, 0x00, 0x00, 0x42, 0x47, 0x07, 0x40, 0x00, 0x40, 0x11, 0x6e, 0xcc, 0xc0, 0xa8, 0x01, 0x89, 0xc0, 0xa8, 0x01, 0xfe, 0xba, 0x2f, 0x00, 0x35, 0x00, 0x2e, 0x1d, 0xf8, 0xbc, 0x81, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x61, 0x70, 0x69, 0x0c, 0x73, 0x74, 0x65, 0x61, 0x6d, 0x70, 0x6f, 0x77, 0x65, 0x72, 0x65, 0x64, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x1c, 0x00, 0x01];

		let ip  = ip::v4::Packet::new(&raw[..]).unwrap();
		let udp = udp::Packet::new(ip.payload()).unwrap();

		assert!(ip.is_valid());
		assert!(udp.is_valid(&ip::Packet::from(&ip)));

		assert_eq!(udp.destination(), 53);
	}
}