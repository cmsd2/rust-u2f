pub mod messages;

use futures::Future;
use self::messages::*;

error_chain! {
    errors {
        OtherError(msg: Option<String>) {
            description("other error")
            display("other error: {:?}", msg)
        }

        BadRequest(msg: Option<String>) {
            description("bad request")
            display("bad request: {:?}", msg)
        }

        ConfigurationUnsupported(msg: Option<String>) {
            description("configuration unsupported")
            display("configuration unsupported: {:?}", msg)
        }

        DeviceIneligible(msg: Option<String>) {
            description("device ineligible")
            display("device ineligible: {:?}", msg)
        }

        Timeout(msg: Option<String>) {
            description("timeout")
            display("timeout: {:?}", msg)
        }
    }
}

#[derive(Clone, Debug)]
pub struct RegisterResponse {
    pub version: String,
    pub registration_data: String,
    pub client_data: String,
}

#[derive(Clone, Debug)]
pub struct SignResponse {
    pub key_handle: String,
    pub signature_data: String,
    pub client_data: String,
}

pub trait U2f {
    fn register(app_id: String, 
        register_requests: Vec<RegisterRequest>, 
        registered_keys: Vec<RegisteredKey>, 
        timeout_seconds: Option<u32>) 
        -> Box<Future<Item=RegisterResponse,Error=Error>>;

    fn sign(app_id: String,
        challenge: String,
        registered_keys: Vec<RegisteredKey>,
        timeout_seconds: Option<u32>)
        -> Box<Future<Item=SignResponse,Error=Error>>;
}
