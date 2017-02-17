use bytebuffer::*;
use raw::error::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum U2fCommand {
    Register = 0x1,
    Authenticate = 0x2,
    Version = 0x3,
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum U2fStatusWord {
    NoError = 0x9000,
    WrongData = 0x6984,
    ConditionsNotSatisfied = 0x6985,
    InsNotSupported = 0x6d00,
    ClaNotSupported = 0x6e00,
}
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandAPDU {
    cla: u8, // reserved by underlying protocol. zero.
    ins: U2fCommand, // u2f command code
    p1: u8, // command parameter 1
    p2: u8, // command parameter 2
    request_data: Vec<u8>,
    le: Option<usize> // max expected length of response data if any [0-256]
}

impl CommandAPDU {
    pub fn new(ins: U2fCommand, p1: u8, p2: u8, request_data: Vec<u8>, le: Option<usize>) -> CommandAPDU {
        CommandAPDU {
            cla: 0,
            ins: ins,
            p1: p1,
            p2: p2,
            request_data: request_data,
            le: le
        }
    }
}

pub trait RequestEncoder {
    fn encode(bb: &mut ByteBuffer, cmd: CommandAPDU) -> Result<()>;

    fn max_request_data() -> usize;

    fn max_response_data() -> usize;
}

pub struct ShortEncoder;

impl RequestEncoder for ShortEncoder {
    fn encode(bb: &mut ByteBuffer, cmd: CommandAPDU) -> Result<()> {
        if cmd.request_data.len() > Self::max_request_data() {
            return Err(ErrorKind::RequestDataTooLong.into());
        }

        if let Some(le) = cmd.le {
            if le > Self::max_response_data() {
                return Err(ErrorKind::ExpectedResponseDataTooLong.into());
            }

            if le == 0 {
                return Err(ErrorKind::ExpectedZeroResponseData.into());
            }
        }

        bb.write_u8(cmd.cla);
        bb.write_u8(cmd.ins as u8);
        bb.write_u8(cmd.p1);
        bb.write_u8(cmd.p2);

        let nc = cmd.request_data.len();
        if nc != 0 {
            bb.write_u8(nc as u8);

            bb.write_bytes(&cmd.request_data[..]);
        }

        if let Some(le) = cmd.le {
            let ne = if le == Self::max_response_data() { 0 } else { le };

            bb.write_u8(ne as u8);
        }

        Ok(())
    }

    fn max_request_data() -> usize {
        255
    }

    fn max_response_data() -> usize {
        256
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum U2fExtendedEncodingVersion {
    V1,
    V1_1
}

pub struct ExtendedEncoderV1;

impl RequestEncoder for ExtendedEncoderV1 {
    fn encode(bb: &mut ByteBuffer, cmd: CommandAPDU) -> Result<()> {
        ExtendedEncoder::extended_encode(bb, cmd, U2fExtendedEncodingVersion::V1)
    }

    fn max_request_data() -> usize {
        ExtendedEncoder::max_request_data()
    }

    fn max_response_data() -> usize {
        ExtendedEncoder::max_response_data()
    }
}

pub struct ExtendedEncoderV1_1;

impl RequestEncoder for ExtendedEncoderV1_1 {
    fn encode(bb: &mut ByteBuffer, cmd: CommandAPDU) -> Result<()> {
        ExtendedEncoder::extended_encode(bb, cmd, U2fExtendedEncodingVersion::V1_1)
    }

    fn max_request_data() -> usize {
        ExtendedEncoder::max_request_data()
    }

    fn max_response_data() -> usize {
        ExtendedEncoder::max_response_data()
    }
}

pub struct ExtendedEncoder;

impl ExtendedEncoder {
    fn extended_encode(bb: &mut ByteBuffer, cmd: CommandAPDU, version: U2fExtendedEncodingVersion) -> Result<()> {
        if cmd.request_data.len() > Self::max_request_data() {
            return Err(ErrorKind::RequestDataTooLong.into());
        }

        if let Some(le) = cmd.le {
            if le > Self::max_response_data() {
                return Err(ErrorKind::ExpectedResponseDataTooLong.into());
            }

            if le == 0 {
                return Err(ErrorKind::ExpectedZeroResponseData.into());
            }
        }

        bb.write_u8(cmd.cla);
        bb.write_u8(cmd.ins as u8);
        bb.write_u8(cmd.p1);
        bb.write_u8(cmd.p2);

        let nc = cmd.request_data.len();

        if version == U2fExtendedEncodingVersion::V1 {
            bb.write_u8(((nc >> 16) & 0xff) as u8);
            bb.write_u8(((nc >> 8) & 0xff) as u8);
            bb.write_u8((nc & 0xff) as u8);
        } else {
            if nc != 0 {
                bb.write_u8(0);
                bb.write_u8(((nc >> 8) & 0xff) as u8);
                bb.write_u8((nc & 0xff) as u8);
            }
        }

        bb.write_bytes(&cmd.request_data[..]);

        if version == U2fExtendedEncodingVersion::V1_1 {
            if let Some(le) = cmd.le {
                let ne = if le == Self::max_response_data() { 0 } else { le };

                if nc == 0 {
                    bb.write_u8(0);
                }

                bb.write_u8(((ne >> 8) & 0xff) as u8);
                bb.write_u8((ne & 0xff) as u8);
            }
        }

        Ok(())
    }

    fn max_request_data() -> usize {
        65535
    }

    fn max_response_data() -> usize {
        65536
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResponseAPDU {
    pub response_data: Vec<u8>,
    pub status: u16
}

impl ResponseAPDU {
    pub fn new(data: Vec<u8>, status: u16) -> ResponseAPDU {
        ResponseAPDU {
            response_data: data,
            status: status
        }
    }
}

pub trait ResponseDecoder {
    fn decode(bb: &mut ByteBuffer) -> Result<ResponseAPDU>;
}

pub struct Decoder;

impl ResponseDecoder for Decoder {
    fn decode(bb: &mut ByteBuffer) -> Result<ResponseAPDU> {
        let bb_len = bb.len();

        if bb_len < 2 {
            return Err(ErrorKind::ResponseFrameTooShort.into());
        }

        let data = bb.read_bytes(bb_len - 2);
        let sw1 = bb.read_u8();
        let sw2 = bb.read_u8();

        let sw = ((sw1 as u16) << 8) | (sw2 as u16);

        Ok(ResponseAPDU::new(data, sw))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_short_get_version() {
        let expected = vec![0, 3, 0, 0, 0];
        let mut bb = ByteBuffer::new();
        ShortEncoder::encode(&mut bb, CommandAPDU::new(U2fCommand::Version, 0, 0, vec![], Some(256))).unwrap();
        assert_eq!(bb.to_bytes(), expected);
    }

    #[test]
    fn test_extended_get_version() {
        let expected = vec![0, 3, 0, 0, 0, 0, 0];
        let mut bb = ByteBuffer::new();
        ExtendedEncoder::encode(&mut bb, CommandAPDU::new(U2fCommand::Version, 0, 0, vec![], Some(65536))).unwrap();
        assert_eq!(bb.to_bytes(), expected);
    }

}