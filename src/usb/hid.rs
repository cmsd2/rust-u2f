use bytebuffer::*;
use super::error::*;
use std::cmp;
use hidapi::*;
use rand;
use rand::Rng;
use enum_primitive::FromPrimitive;

#[derive(Debug, Clone)]
pub struct U2fDeviceInfo {
    pub protocol_version: u8,
    pub major_device_version: u8,
    pub minor_device_version: u8,
    pub build_device_version: u8,
    pub raw_capabilities: u8,
}

pub struct U2fDevice<'a> {
    pub packet_size: usize,
    pub channel_id: u32,
    pub hid_device: HidDevice<'a>,
    pub u2f_info: Option<U2fDeviceInfo>,
}

pub const BROADCAST_CID: u32 = 0xffffffff;
pub const HID_REPORT_SIZE: usize = 64;
pub const FIDO_USAGE_PAGE: u16 = 0xf1d0;
pub const U2F_USAGE: u16 = 0x1;

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum U2fHidCommand {
    Ping = 0x81,
    Msg = 0x83,
    Lock = 0x84,
    Init = 0x86,
    Wink = 0x88,
    Error = 0xbf,
}
}

impl <'a> U2fDevice<'a> {
    pub fn ping(&self) -> Result<()> {
        let mut buf = ByteBuffer::new();
        buf.write_u8(0);

        self.command(U2fHidCommand::Ping, &mut buf)?;

        println!("read {} bytes", buf.len());

        Ok(())
    }

    pub fn wink(&self) -> Result<()> {
        let mut buf = ByteBuffer::new();

        self.command(U2fHidCommand::Wink, &mut buf);

        Ok(())
    }

    pub fn message(&self, msg: &[u8]) -> Result<Vec<u8>> {
        let mut buf = ByteBuffer::new();

        buf.write_bytes(msg);

        self.command(U2fHidCommand::Msg, &mut buf);

        Ok(buf.to_bytes())
    }

    pub fn nonce() -> Vec<u8> {
        let mut rng = rand::thread_rng();
        
        let mut nonce = vec![0;8];

        for x in nonce.iter_mut() {
            *x = rng.gen();
        }

        nonce
    }

    pub fn init(&mut self) -> Result<()> {
        self.channel_id = BROADCAST_CID;

        let mut buf = ByteBuffer::new();
        
        let nonce = Self::nonce();
        buf.write_bytes(nonce.as_slice());

        self.send_request(U2fHidCommand::Init, &mut buf)?;
        
        buf.clear();

        loop {
            self.recv_response(U2fHidCommand::Init, &mut buf)?;

            if buf.len() < 17 {
                bail!(ErrorKind::InitResponseTooSmall);
            }

            let recvd_nonce = buf.read_bytes(8);
            if recvd_nonce != nonce {
                continue;
            }

            self.channel_id = buf.read_u32();

            let info = U2fDeviceInfo {
                protocol_version: buf.read_u8(),
                major_device_version: buf.read_u8(),
                minor_device_version: buf.read_u8(),
                build_device_version: buf.read_u8(),
                raw_capabilities: buf.read_u8(),
            };

            self.u2f_info = Some(info);

            break;
        }

        Ok(())
    }

    pub fn command(&self, command: U2fHidCommand, buf: &mut ByteBuffer) -> Result<()> {
        self.send_request(command, buf)?;

        buf.clear();

        self.recv_response(command, buf)?;

        Ok(())
    }

    pub fn send_request(&self, command: U2fHidCommand, request_data: &mut ByteBuffer) -> Result<()> {
        if (request_data.len() - request_data.get_rpos()) >= 7609 {
            bail!(ErrorKind::RequestTooLong);
        }

        let mut request = ByteBuffer::new();

        request.write_u8(0x0); // hid report number

        prepare_init_packet(&mut request, self.channel_id, command, request_data, self.packet_size);

        println!("sending {} bytes", request.len());
        println!("sending {:?}", request.to_bytes());

        // send init packet
        self.hid_device.write(&request.to_bytes()[..])?;

        let mut seq: u8 = 0;

        while request_data.get_rpos() < request_data.len() {
            request.clear();
            seq += 1;

            request.write_u8(0x0); // hid report number

            prepare_cont_packet(&mut request, self.channel_id, seq, request_data, self.packet_size);

            // send cont packet
            self.hid_device.write(&request.to_bytes()[..])?;
        }

        Ok(())
    }

    pub fn recv_response(&self, command: U2fHidCommand, response: &mut ByteBuffer) -> Result<()> {    
        let mut data = ByteBuffer::new();

        // read init packet

        let mut report = vec![0; HID_REPORT_SIZE];
        let bytes = self.hid_device.read_timeout(report.as_mut_slice(), 3000 /* millis */)?;
        println!("read {} bytes", bytes);
        println!("read {:?}", report.as_slice());
        data.write_bytes(&report[0..bytes]);

        let init_frame = parse_init_packet(&mut data, self.packet_size)?;

        if init_frame.channel_id != self.channel_id {
            return Err(ErrorKind::UnknownChannelId.into());
        }

        let mut payload_remaining = init_frame.len;
        let fragment = &init_frame.payload[..];
        let fragment_len = cmp::min(fragment.len(), payload_remaining);

        response.write_bytes(&fragment[0..fragment_len]);
        payload_remaining -= fragment_len;

        while payload_remaining != 0 {
            data.clear();

            // read cont packet

            let bytes = self.hid_device.read_timeout(report.as_mut_slice(), 3000 /* millis */)?;
            println!("read {} bytes", bytes);
            println!("read {:?}", report.as_slice());
            data.write_bytes(&report[0..bytes]);

            let frame = parse_cont_packet(&mut data, self.packet_size)?;

            let fragment = &frame.payload[..];
            let fragment_len = cmp::min(fragment.len(), payload_remaining);
            
            response.write_bytes(&fragment[0..fragment_len]);
            payload_remaining -= fragment_len;
        }

        if U2fHidCommand::from_u8(init_frame.command) != Some(command) {
            bail!(ErrorKind::HidError);
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct HidInitPacket {
    pub channel_id: u32,
    pub command: u8, // command byte. high bit set.
    pub len: usize, // payload length before fragmentation
    pub payload: Vec<u8>, // first fragment of payload maybe with trailing 0 padding
}

#[derive(Debug, Clone)]
pub struct HidContPacket {
    pub channel_id: u32,
    pub seq: u8, // sequence number of continuation packet in range [0,127]
    pub payload: Vec<u8>, // continuation fragment of payload maybe with trailing 0 padding
}

#[derive(Debug, Clone)]
pub enum HidPacket {
    Init(HidInitPacket),
    Cont(HidContPacket)
}

pub fn parse_init_packet(data: &mut ByteBuffer, frame_size: usize) -> Result<HidInitPacket> {
    match parse_packet(data, frame_size)? {
        HidPacket::Init(p) => Ok(p),
        _ => Err(ErrorKind::UnexpectedPacket.into())
    }
}

pub fn parse_cont_packet(data: &mut ByteBuffer, frame_size: usize) -> Result<HidContPacket> {
    match parse_packet(data, frame_size)? {
        HidPacket::Cont(p) => Ok(p),
        _ => Err(ErrorKind::UnexpectedPacket.into())
    }
}

pub fn parse_packet(data: &mut ByteBuffer, frame_size: usize) -> Result<HidPacket> {
    if frame_size <= 7 {
        return Err(ErrorKind::HidPacketTooSmall.into());
    }

    if data.len() - data.get_rpos() < frame_size {
        return Err(ErrorKind::HidPacketTooSmall.into());
    }

    let channel_id = data.read_u32();
    let command_or_seq = data.read_u8();

    if (command_or_seq & 0x80) != 0 {
        let command = command_or_seq;

        let len1 = data.read_u8() as usize;
        let len2 = data.read_u8() as usize;
        let len = (len1 << 8) | len2;

        let payload = data.read_bytes(frame_size - 7);

        Ok(HidPacket::Init(HidInitPacket {
            channel_id: channel_id,
            command: command,
            len: len,
            payload: payload,
        }))
    } else {
        let seq = command_or_seq;

        let payload = data.read_bytes(frame_size - 5);

        Ok(HidPacket::Cont(HidContPacket {
            channel_id: channel_id,
            seq: seq,
            payload: payload,
        }))
    }
}

pub fn prepare_init_packet(request: &mut ByteBuffer, channel_id: u32, command: U2fHidCommand, data: &mut ByteBuffer, packet_len: usize) {
    request.write_u32(channel_id);
    request.write_u8(command as u8);

    let data_len = data.len() - data.get_rpos();
    request.write_u8(((data_len >> 8) & 0xff) as u8);
    request.write_u8((data_len & 0xff) as u8);

    let copied = copy_buffer(data, request, packet_len - 7);
    pad_buffer(request, packet_len - 7 - copied);
}

pub fn prepare_cont_packet(request: &mut ByteBuffer, channel_id: u32, counter: u8, data: &mut ByteBuffer, packet_len: usize) {
    request.write_u32(channel_id);
    request.write_u8(counter);

    let copied = copy_buffer(data, request, packet_len - 5);
    pad_buffer(request, packet_len - 5 - copied);
}

pub fn copy_buffer(src: &mut ByteBuffer, dest: &mut ByteBuffer, max_len: usize) -> usize {
    let available = src.len() - src.get_rpos();
    let count = cmp::min(available, max_len);

    for _i in 0..count {
        dest.write_u8(src.read_u8());
    }

    count
}

pub fn pad_buffer(buf: &mut ByteBuffer, extra: usize) {
    for _i in 0..extra {
        buf.write_u8(0);
    }
}