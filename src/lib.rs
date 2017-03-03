extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate futures;
#[macro_use]
extern crate error_chain;
extern crate bytebuffer;
#[macro_use]
extern crate enum_primitive;
extern crate hidapi;
extern crate rand;
extern crate webpki;
extern crate untrusted;
extern crate owning_ref;
extern crate lifeguard;

#[macro_use]
pub mod serde_enum;

pub mod api;
pub mod raw;
pub mod usb;
pub mod error;

use std::cell::RefCell;
use bytebuffer::*;
use usb::hid::*;
use raw::frame::*;
use error::*;
use owning_ref::*;
use lifeguard::*;

pub const TEST_USER_PRESENCE_REQUIRED: u8 = 1;
pub const TEST_USER_PRESENCE_CONSUME: u8 = 2;
pub const TEST_USER_PRESENCE_TEST_ONLY: u8 = 4;

pub const AUTH_USER_PRESENCE_ENFORCE: u8 = TEST_USER_PRESENCE_REQUIRED | TEST_USER_PRESENCE_CONSUME;
pub const AUTH_USER_PRESENCE_CHECK: u8 = TEST_USER_PRESENCE_REQUIRED | TEST_USER_PRESENCE_CONSUME | TEST_USER_PRESENCE_TEST_ONLY;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum U2fVersion {
    V2
}

pub trait Signature {
    fn user_public_key<'a>(&'a self) -> &'a [u8];
    fn key_handle<'a>(&'a self) -> &'a [u8];
    fn cert<'a>(&'a self) -> Result<webpki::EndEntityCert<'a>>;
    fn signature<'a>(&'a self) -> &'a [u8];
}

pub trait Verify {
    fn verify(&self, challenge_param: &[u8], app_param: &[u8]) -> Result<()>;
}

impl <T> Verify for T where T: Signature {
    fn verify(&self, challenge_param: &[u8], app_param: &[u8]) -> Result<()> {
        let mut msg = ByteBuffer::new();
        msg.write_u8(0);
        msg.write_bytes(app_param);
        msg.write_bytes(challenge_param);
        msg.write_bytes(self.key_handle());
        msg.write_bytes(self.user_public_key());
        let signing_string = msg.to_bytes();

        let cert = self.cert().expect("cert");

        cert.verify_signature(&webpki::ECDSA_P256_SHA256, 
            untrusted::Input::from(&signing_string),
            untrusted::Input::from(&self.signature()))
            .map_err(|e| ErrorKind::WebPkiError(e).into())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user_public_key: Vec<u8>,
    pub key_handle: Vec<u8>,
    pub attestation_cert: Vec<u8>,
    pub signature: Vec<u8>,
}

impl Signature for RegisterResponse {
    fn key_handle<'a>(&'a self) -> &'a [u8] {
        &self.key_handle
    }

    fn user_public_key<'a>(&'a self) -> &'a [u8] {
        &self.user_public_key
    }

    fn cert<'a>(&'a self) -> Result<webpki::EndEntityCert<'a>> {
        parse_cert(&self.attestation_cert).into()
    }

    fn signature<'a>(&'a self) -> &'a [u8] {
        &self.signature
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthenticateResponse {
    pub counter: u32,
    pub signature: Vec<u8>,
}

pub trait U2fDevice {
    fn register<'b>(&self, challenge_param: &[u8], application_param: &[u8]) -> Result<RegisterResponse>;

    fn authenticate(&self, challenge_param: &[u8], application_param: &[u8], key_handle: &[u8]) -> Result<AuthenticateResponse>;

    fn get_version(&self) -> Result<U2fVersion>;
}

impl <'a> U2fDevice for U2fHidDevice<'a> {
    fn register<'b>(&self, challenge_param: &[u8], application_param: &[u8]) -> Result<RegisterResponse> {
        if challenge_param.len() != 32 {
            bail!(ErrorKind::InvalidChallengeParameter);
        }

        if application_param.len() != 32 {
            bail!(ErrorKind::InvalidApplicationParameter);
        }

        let mut buf = ByteBuffer::new();

        buf.write_bytes(challenge_param);
        buf.write_bytes(application_param);

        let response = match self.send_apdu::<ExtendedEncoderV1>(
                CommandAPDU::new(U2fCommand::Register, AUTH_USER_PRESENCE_ENFORCE, 0, buf.to_bytes(), Some(256))) {
            Ok(response) => response,
            Err(usb::error::Error(usb::error::ErrorKind::ErrorStatus(U2fStatusWord::ConditionsNotSatisfied), _)) => bail!(ErrorKind::UserPresenceRequired),
            Err(e) => bail!(e)
        };

        buf = ByteBuffer::from_bytes(&response.response_data[..]);

        let reserved = buf.read_u8();
        if reserved != 0x05 {
            bail!(ErrorKind::InvalidRegistrationResponse);
        }
        let public_key = buf.read_bytes(65);
        let key_handle_len = buf.read_u8() as usize;
        if(buf.len() - buf.get_rpos()) <= key_handle_len {
            bail!(ErrorKind::InvalidRegistrationResponse);
        }
        let key_handle = buf.read_bytes(key_handle_len);

        let remaining_len = buf.len() - buf.get_rpos();
        let remaining = buf.read_bytes(remaining_len);

        let cert_len = cert_len(&remaining[..]);
        let cert_bytes = remaining[0..cert_len].to_owned();
        let signature = remaining[cert_len..].to_owned();
        
        Ok(RegisterResponse {
            user_public_key: public_key,
            key_handle: key_handle,
            attestation_cert: cert_bytes,
            signature: signature
        })
    }

    fn authenticate(&self, challenge_param: &[u8], application_param: &[u8], key_handle: &[u8]) -> Result<AuthenticateResponse> {
        if challenge_param.len() != 32 {
            bail!(ErrorKind::InvalidChallengeParameter);
        }

        if application_param.len() != 32 {
            bail!(ErrorKind::InvalidApplicationParameter);
        }

        if key_handle.len() >= 256 {
            bail!(ErrorKind::KeyHandleTooLong);
        }

        let mut buf = ByteBuffer::new();

        buf.write_bytes(challenge_param);
        buf.write_bytes(application_param);
        buf.write_u8(key_handle.len() as u8);
        buf.write_bytes(key_handle);

        let response = match self.send_apdu::<ExtendedEncoderV1>(
                CommandAPDU::new(U2fCommand::Authenticate, AUTH_USER_PRESENCE_ENFORCE, 0, buf.to_bytes(), Some(256))) {
            Ok(response) => response,
            Err(usb::error::Error(usb::error::ErrorKind::ErrorStatus(U2fStatusWord::ConditionsNotSatisfied), _)) => bail!(ErrorKind::UserPresenceRequired),
            Err(e) => bail!(e)
        };

        buf = ByteBuffer::from_bytes(response.response_data.as_slice());
        
        buf.read_u8(); // user presence
        let counter = buf.read_u32();
        let signature_len = buf.len() - buf.get_rpos();
        let signature = buf.read_bytes(signature_len);

        Ok(AuthenticateResponse {
            counter: counter,
            signature: signature
        })
    }

    fn get_version(&self) -> Result<U2fVersion> {
        println!("sending version command");

        let response = self.send_apdu::<ExtendedEncoderV1>(CommandAPDU::new(U2fCommand::Version, 0, 0, vec![], Some(256)))?;

        if response.response_data == "U2F_V2".as_bytes() {
            Ok(U2fVersion::V2)
        } else {
            Err(ErrorKind::UnrecognisedVersion.into())
        }
    }
}

pub fn owning_bytes(bytes: Vec<u8>) -> OwningRef<Vec<u8>, [u8]> {
    OwningRef::new(bytes)
}

pub fn parse_cert_static(bytes: Vec<u8>) -> Result<OwningHandle<Vec<u8>, Box<webpki::EndEntityCert<'static>>>> {
    OwningHandle::try_new(bytes, |ptr| {
        Ok(Box::new(parse_cert(unsafe { &*ptr })?))
    })
}

pub fn parse_cert<'a>(bytes: &'a [u8]) -> Result<webpki::EndEntityCert<'a>> {
    let input = untrusted::Input::from(bytes);
    webpki::EndEntityCert::from(input).map_err(|e| ErrorKind::WebPkiError(e).into())
}

pub fn cert_len(bytes: &[u8]) -> usize {
    let input = untrusted::Input::from(&bytes[..]);
    let mut reader = untrusted::Reader::new(input);

    let mark1 = reader.mark();
    let cert = webpki::EndEntityCert::from_reader(&mut reader);
    let mark2 = reader.mark();

    let cert_input = reader.get_input_between_marks(mark1, mark2).unwrap();

    cert_input.len()
}