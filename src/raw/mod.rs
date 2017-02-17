pub mod frame;
pub mod error;

use bytebuffer::*;
use self::frame::*;
use self::error::*;
use super::usb::hid::*;
use enum_primitive::FromPrimitive;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum U2fVersion {
    V2
}

pub fn send_apdu<E>(device: &U2fDevice, cmd: CommandAPDU) -> Result<ResponseAPDU> where E: RequestEncoder {
    let mut bb = ByteBuffer::new();

    E::encode(&mut bb, cmd)?;

    println!("encoded {} bytes", bb.len());

    device.command(U2fHidCommand::Msg, &mut bb)?;

    Decoder::decode(&mut bb)
}

pub fn get_version(device: &U2fDevice) -> Result<U2fVersion> {
    println!("sending version command");

    let response = send_apdu::<ExtendedEncoderV1>(device, CommandAPDU::new(U2fCommand::Version, 0, 0, vec![], Some(256)))?;

    match U2fStatusWord::from_u16(response.status) {
        Some(U2fStatusWord::NoError) => {
            if response.response_data == "U2F_V2".as_bytes() {
                Ok(U2fVersion::V2)
            } else {
                Err(ErrorKind::UnrecognisedVersion.into())
            }
        }
        Some(err) => {
            Err(ErrorKind::ErrorStatus(response.status).into())
        }
        None => {
            Err(ErrorKind::ErrorStatus(response.status).into())
        }
    }
}