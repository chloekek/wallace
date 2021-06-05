#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::format_bytes_16;
use super::format_u16;
use super::format_u32;
use super::format_u64;

/// [MS-SMB2] section 2.2.4.
#[derive(Clone, Copy, Debug)]
pub struct SMB2_NEGOTIATE_Response
{
    pub StructureSize:          u16,
    pub SecurityMode:           u16,
    pub DialectRevision:        u16,
    pub NegotiateContextCount:  u16,
    pub ServerGuid:             [u8; 16],
    pub Capabilities:           u32,
    pub MaxTransactSize:        u32,
    pub MaxReadSize:            u32,
    pub MaxWriteSize:           u32,
    pub SystemTime:             u64,
    pub ServerStartTime:        u64,
    pub SecurityBufferOffset:   u16,
    pub SecurityBufferLength:   u16,
    pub NegotiateContextOffset: u32,
}

impl SMB2_NEGOTIATE_Response
{
    pub fn format(&self, into: &mut Vec<u8>)
    {
        format_u16(into, self.StructureSize);
        format_u16(into, self.SecurityMode);
        format_u16(into, self.DialectRevision);
        format_u16(into, self.NegotiateContextCount);
        format_bytes_16(into, self.ServerGuid);
        format_u32(into, self.Capabilities);
        format_u32(into, self.MaxTransactSize);
        format_u32(into, self.MaxReadSize);
        format_u32(into, self.MaxWriteSize);
        format_u64(into, self.SystemTime);
        format_u64(into, self.ServerStartTime);
        format_u16(into, self.SecurityBufferOffset);
        format_u16(into, self.SecurityBufferLength);
        format_u32(into, self.NegotiateContextOffset);
    }
}
