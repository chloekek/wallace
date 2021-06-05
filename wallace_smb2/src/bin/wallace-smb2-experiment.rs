use std::io::Read;
use std::io::Result;
use std::io::Write;
use std::net::TcpListener;
use wallace_smb2 as smb2;

fn main() -> Result<()>
{
    let tcp_listener = TcpListener::bind("127.1:5000")?;
    let (mut tcp_stream, _) = tcp_listener.accept()?;

    {
        let packet_header =
            smb2::Direct_TCP_transport_packet_header::read(&mut tcp_stream)?;

        let mut packet = vec![0; packet_header.StreamProtocolLength as usize];
        tcp_stream.read_exact(&mut packet)?;

        let mut input = &packet[..];

        let header = smb2::SMB2_Packet_Header::parse(&mut input);
        eprintln!("{:?}", header);

        let request = smb2::SMB2_NEGOTIATE_Request::parse(&mut input);
        eprintln!("{:?}", request);
    }

    {
        let header = smb2::SMB2_Packet_Header{
            ProtocolId:                      0x424D53FE,
            StructureSize:                   64,
            CreditCharge:                    0,
            ChannelSequence_Reserved_Status: 0,
            Command:
                smb2::SMB2_Packet_Header::SMB2_NEGOTIATE,
            CreditRequest_CreditResponse:    31,
            Flags:
                smb2::SMB2_Packet_Header::SMB2_FLAGS_SERVER_TO_REDIR,
            NextCommand:                     0,
            MessageId:                       1,
            AsyncId_Reserved_TreeId:         0,
            SessionId:                       0,
            Signature:                       [0; 16],
        };

        let response = smb2::SMB2_NEGOTIATE_Response{
            StructureSize:          65,
            SecurityMode:           0,
            DialectRevision:        0x0210,
            NegotiateContextCount:  0,
            ServerGuid:             [0x4B; 16],
            Capabilities:           0,
            MaxTransactSize:        4096,
            MaxReadSize:            4096,
            MaxWriteSize:           4096,
            SystemTime:             13_267_313_015 * 10_000_000,
            ServerStartTime:        13_267_313_015 * 10_000_000,
            SecurityBufferOffset:   64 + 65,
            SecurityBufferLength:   16,
            NegotiateContextOffset: 0,
        };

        let mut packet = Vec::new();
        header.format(&mut packet);
        response.format(&mut packet);
        packet.extend(&[0; 16]);

        let packet_header = smb2::Direct_TCP_transport_packet_header{
            Zero: 0,
            StreamProtocolLength: packet.len() as u32,
        };

        packet_header.write(&mut tcp_stream)?;
        tcp_stream.write(&packet)?;
        tcp_stream.flush()?;

        packet_header.write(&mut std::io::stdout())?;
        std::io::stdout().write(&packet)?;
        std::io::stdout().flush()?;
    }

    Ok(())
}
