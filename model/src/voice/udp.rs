use byteorder::{WriteBytesExt,ReadBytesExt,NetworkEndian,LittleEndian};
use std::{
    net::IpAddr,
    io::Write,
};
use failure::Fail;

pub const SAMPLE_RATE: u32 = 48000;

pub const RTP_HEADER_LEN: usize = 12;

pub fn discovery_request(mut buf: [u8;70], ssrc: u32) -> Result<[u8;70],std::io::Error>{
    (&mut buf[..]).write_u32::<NetworkEndian>(ssrc)?;
    Ok(buf)
}

#[derive(Debug,Fail)]
pub enum DiscoveryPacketError{
    #[fail(display = "Udp ip discovery response was not of the correct format")]
    BadPacketFormat,
    #[fail(display = "Ip discovery repsonse ip address was not utf8")]
    IpNotUtf8,
    #[fail(display = "Ip discovery repsonse ip address was not a valid ip address")]
    BadIp,
}

pub fn parse_discovery_response(buf: [u8;70]) -> Result<(IpAddr,u16),DiscoveryPacketError>{
    //first 4 bytes are subscriber
    //ip follows as null terminated string
    //last 2 bytes are port as u16le

    let ip_len = buf.iter().skip(4).position(|b| *b == 0).ok_or_else(|| DiscoveryPacketError::BadPacketFormat)?;
    let ip_str = String::from_utf8(buf[4..4+ip_len].to_vec()).map_err(|_| DiscoveryPacketError::IpNotUtf8)?;

    let ip: std::net::IpAddr = ip_str.parse().map_err(|_| DiscoveryPacketError::BadIp)?;

    let port = (&buf[buf.len()-3..]).read_u16::<LittleEndian>().map_err(|_| DiscoveryPacketError::BadPacketFormat)?;

    Ok((ip,port))
}

pub fn rtp_header(mut packet_buf: &mut [u8], seq_num: u16, timestamp: u32, ssrc: u32) -> Result<(),std::io::Error>
{
    packet_buf.write_all(&[0x80, 0x78])?;
    packet_buf.write_u16::<NetworkEndian>(seq_num)?;
    packet_buf.write_u32::<NetworkEndian>(timestamp)?;
    packet_buf.write_u32::<NetworkEndian>(ssrc)?;
    Ok(())
}

pub fn nonce(packet: &[u8]) -> [u8;24]
{
    assert!(packet.len() >= RTP_HEADER_LEN);
    let mut nonce = [0u8;24];
    nonce[..RTP_HEADER_LEN].copy_from_slice(&packet[..RTP_HEADER_LEN]);
    nonce
}

#[cfg(test)]
mod test{
    #[test]
    fn header_sanity_check(){
        let mut packet = [0u8;super::RTP_HEADER_LEN];
        assert!(super::rtp_header(&mut packet, 0xFF00, 0xFF_00_FF_00, 0xFF_00_FF_00).is_ok());
        assert_eq!(&packet,&[0x80u8,0x78,0xFF,0x00,0xFF,0x00,0xFF,0x00,0xFF,0x00,0xFF,0x00])
    }
}