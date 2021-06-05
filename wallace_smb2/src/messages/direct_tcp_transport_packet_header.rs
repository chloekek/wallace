#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::io::Read;
use std::io::Result;
use std::io::Write;

/// [MS-SMB2] section 2.1.
pub struct Direct_TCP_transport_packet_header
{
    pub Zero:                 u8,
    pub StreamProtocolLength: u32,
}

impl Direct_TCP_transport_packet_header
{
    pub fn read(reader: &mut impl Read) -> Result<Self>
    {
        let mut buf = [0; 4];
        reader.read_exact(&mut buf)?;

        let Zero = buf[0];
        let StreamProtocolLength =
            u32::from_be_bytes([0, buf[1], buf[2], buf[3]]);

        Ok(Self{Zero, StreamProtocolLength})
    }

    pub fn write(&self, writer: &mut impl Write) -> Result<()>
    {
        let buf = self.StreamProtocolLength.to_be_bytes();
        let buf = [self.Zero, buf[1], buf[2], buf[3]];
        writer.write_all(&buf)
    }
}
