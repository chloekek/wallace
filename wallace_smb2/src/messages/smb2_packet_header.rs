#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::format_bytes_16;
use super::format_u16;
use super::format_u32;
use super::format_u64;
use super::parse_bytes_16;
use super::parse_u16;
use super::parse_u32;
use super::parse_u64;

/// [MS-SMB2] section 2.2.1.
#[derive(Clone, Copy, Debug)]
pub struct SMB2_Packet_Header
{
    pub ProtocolId:                      u32,
    pub StructureSize:                   u16,
    pub CreditCharge:                    u16,
    pub ChannelSequence_Reserved_Status: u32,
    pub Command:                         u16,
    pub CreditRequest_CreditResponse:    u16,
    pub Flags:                           u32,
    pub NextCommand:                     u32,
    pub MessageId:                       u64,
    pub AsyncId_Reserved_TreeId:         u64,
    pub SessionId:                       u64,
    pub Signature:                       [u8; 16],
}

impl SMB2_Packet_Header
{
    pub const SMB2_NEGOTIATE: u16 = 0x0000;

    pub const SMB2_FLAGS_SERVER_TO_REDIR: u32 = 0x00000001;

    pub fn parse(i: &mut &[u8]) -> Option<Self>
    {
        let ProtocolId                      = parse_u32(i)?;
        let StructureSize                   = parse_u16(i)?;
        let CreditCharge                    = parse_u16(i)?;
        let ChannelSequence_Reserved_Status = parse_u32(i)?;
        let Command                         = parse_u16(i)?;
        let CreditRequest_CreditResponse    = parse_u16(i)?;
        let Flags                           = parse_u32(i)?;
        let NextCommand                     = parse_u32(i)?;
        let MessageId                       = parse_u64(i)?;
        let AsyncId_Reserved_TreeId         = parse_u64(i)?;
        let SessionId                       = parse_u64(i)?;
        let Signature                       = parse_bytes_16(i)?;
        Some(
            Self{
                ProtocolId,
                StructureSize,
                CreditCharge,
                ChannelSequence_Reserved_Status,
                Command,
                CreditRequest_CreditResponse,
                Flags,
                NextCommand,
                MessageId,
                AsyncId_Reserved_TreeId,
                SessionId,
                Signature,
            }
        )
    }

    pub fn format(&self, into: &mut Vec<u8>)
    {
        format_u32(into, self.ProtocolId);
        format_u16(into, self.StructureSize);
        format_u16(into, self.CreditCharge);
        format_u32(into, self.ChannelSequence_Reserved_Status);
        format_u16(into, self.Command);
        format_u16(into, self.CreditRequest_CreditResponse);
        format_u32(into, self.Flags);
        format_u32(into, self.NextCommand);
        format_u64(into, self.MessageId);
        format_u64(into, self.AsyncId_Reserved_TreeId);
        format_u64(into, self.SessionId);
        format_bytes_16(into, self.Signature);
    }
}
