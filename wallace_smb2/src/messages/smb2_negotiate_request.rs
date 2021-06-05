#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::parse_bytes_16;
use super::parse_u16;
use super::parse_u32;

/// [MS-SMB2] section 2.2.3.
#[derive(Clone, Copy, Debug)]
pub struct SMB2_NEGOTIATE_Request
{
    pub StructureSize:          u16,
    pub DialectCount:           u16,
    pub SecurityMode:           u16,
    pub Reserved:               u16,
    pub Capabilities:           u32,
    pub ClientGuid:             [u8; 16],
    pub NegotiateContextOffset: u32,
    pub NegotiateContextCount:  u16,
    pub Reserved2:              u16,
}

impl SMB2_NEGOTIATE_Request
{
    pub fn parse(i: &mut &[u8]) -> Option<Self>
    {
        let StructureSize          = parse_u16(i)?;
        let DialectCount           = parse_u16(i)?;
        let SecurityMode           = parse_u16(i)?;
        let Reserved               = parse_u16(i)?;
        let Capabilities           = parse_u32(i)?;
        let ClientGuid             = parse_bytes_16(i)?;
        let NegotiateContextOffset = parse_u32(i)?;
        let NegotiateContextCount  = parse_u16(i)?;
        let Reserved2              = parse_u16(i)?;
        Some(
            Self{
                StructureSize,
                DialectCount,
                SecurityMode,
                Reserved,
                Capabilities,
                ClientGuid,
                NegotiateContextOffset,
                NegotiateContextCount,
                Reserved2,
            }
        )
    }
}
