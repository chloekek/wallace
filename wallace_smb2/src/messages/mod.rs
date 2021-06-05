pub use self::direct_tcp_transport_packet_header::*;
pub use self::smb2_negotiate_request::*;
pub use self::smb2_negotiate_response::*;
pub use self::smb2_packet_header::*;

mod direct_tcp_transport_packet_header;
mod smb2_negotiate_request;
mod smb2_negotiate_response;
mod smb2_packet_header;

fn format_bytes(into: &mut Vec<u8>, bytes: &[u8])
{
    into.extend(bytes);
}

fn format_bytes_16(into: &mut Vec<u8>, bytes: [u8; 16])
{
    format_bytes(into, &bytes);
}

fn format_u16(into: &mut Vec<u8>, value: u16)
{
    let bytes = value.to_le_bytes();
    format_bytes(into, &bytes);
}

fn format_u32(into: &mut Vec<u8>, value: u32)
{
    let bytes = value.to_le_bytes();
    format_bytes(into, &bytes);
}

fn format_u64(into: &mut Vec<u8>, value: u64)
{
    let bytes = value.to_le_bytes();
    format_bytes(into, &bytes);
}

fn parse_bytes<'a>(i: &mut &'a [u8], n: usize) -> Option<&'a [u8]>
{
    if i.len() < n { return None; }
    let bytes = &i[0 .. n];
    *i = &i[n ..];
    Some(bytes)
}

fn parse_bytes_16(i: &mut &[u8]) -> Option<[u8; 16]>
{
    let bytes = parse_bytes(i, 16)?;
    Some([bytes[ 0], bytes[ 1], bytes[ 2], bytes[ 3],
          bytes[ 4], bytes[ 5], bytes[ 6], bytes[ 7],
          bytes[ 8], bytes[ 9], bytes[10], bytes[11],
          bytes[12], bytes[13], bytes[14], bytes[15]])
}

fn parse_u16(i: &mut &[u8]) -> Option<u16>
{
    let bytes = parse_bytes(i, 2)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn parse_u32(i: &mut &[u8]) -> Option<u32>
{
    let bytes = parse_bytes(i, 4)?;
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn parse_u64(i: &mut &[u8]) -> Option<u64>
{
    let bytes = parse_bytes(i, 8)?;
    Some(u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3],
                             bytes[4], bytes[5], bytes[6], bytes[7]]))
}
