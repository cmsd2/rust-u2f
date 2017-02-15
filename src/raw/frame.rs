use bytebuffer::*;
use raw::error::*;

pub struct CommandAPDU {
    cla: u8, // reserved by underlying protocol. zero.
    ins: u8, // u2f command code
    p1: u8, // command parameter 1
    p2: u8, // command parameter 2
    request_data: Vec<u8>,
    le: Option<usize> // max expected length of response data if any [0-256]
}

impl CommandAPDU {
    pub fn new(ins: u8, p1: u8, p2: u8, request_data: Vec<u8>, le: Option<usize>) -> CommandAPDU {
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
        bb.write_u8(cmd.ins);
        bb.write_u8(cmd.p1);
        bb.write_u8(cmd.p2);

        let nc = cmd.request_data.len();
        if nc != 0 {
            bb.write_u8(nc as u8);

            bb.write_bytes(&cmd.request_data[..]);
        }

        // section 3.1.1 is weird.
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

pub struct ExtendedEncoder;

impl RequestEncoder for ExtendedEncoder {
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
        bb.write_u8(cmd.ins);
        bb.write_u8(cmd.p1);
        bb.write_u8(cmd.p2);

        let nc = cmd.request_data.len();
        if nc != 0 {
            bb.write_u8(0);
            bb.write_u8(((nc >> 8) & 0xff) as u8);
            bb.write_u8((nc & 0xff) as u8);

            bb.write_bytes(&cmd.request_data[..]);
        }

        if let Some(le) = cmd.le {
            let ne = if le == Self::max_response_data() { 0 } else { le };

            if nc == 0 {
                bb.write_u8(0);
            }

            bb.write_u8(((ne >> 8) & 0xff) as u8);
            bb.write_u8((ne & 0xff) as u8);
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
        ShortEncoder::encode(&mut bb, CommandAPDU::new(3, 0, 0, vec![], Some(256))).unwrap();
        assert_eq!(bb.to_bytes(), expected);
    }

    #[test]
    fn test_extended_get_version() {
        let expected = vec![0, 3, 0, 0, 0, 0, 0];
        let mut bb = ByteBuffer::new();
        ExtendedEncoder::encode(&mut bb, CommandAPDU::new(3, 0, 0, vec![], Some(65536))).unwrap();
        assert_eq!(bb.to_bytes(), expected);
    }

}