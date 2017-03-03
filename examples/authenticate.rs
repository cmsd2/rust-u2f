extern crate u2f;
extern crate hidapi;
extern crate serde_json;
extern crate untrusted;
extern crate webpki;

use u2f::usb::hid::*;
use u2f::usb::*;
use u2f::*;
use hidapi::*;

use std::fs;
use std::path;
use std::io::Read;
use std::io::Write;
use std::time::Duration;

pub fn main() {
    let path = path::Path::new("regresp.json");
    let mut f = fs::File::open(path).expect("open");
    let mut json = String::new();
    f.read_to_string(&mut json).expect("read");
    let reg = serde_json::from_str::<RegisterResponse>(&json).expect("json");

    // get a fresh challenge from remote server..
    let app_param = vec![0;32];
    let challenge_param = vec![0;32];

    

    let api = HidApi::new().unwrap();

    let devices = api.fido_devices();
    
    for device in devices.iter() {
        println!("{:#?}", device);
    }

    if let Some(device) = devices.first() {
        let auth = authenticate(&api, &device, &challenge_param, &app_param, &reg.key_handle);

        write_auth_response_to_file(&auth);
    } else {
        println!("no fido device found");
        std::process::exit(1);
    }
}

pub fn authenticate(api: &HidApi, device_info: &HidDeviceInfo, challenge_param: &[u8], app_param: &[u8], key_handle: &[u8]) -> AuthenticateResponse {
    let hid_device = api.open_path(&device_info.path).expect("open");

    let mut device = U2fHidDevice {
        packet_size: 64,
        channel_id: BROADCAST_CID,
        hid_device: hid_device,
        u2f_info: None,
    };

    device.init().expect("init");

    println!("device initialised: chan={} {:?}", device.channel_id, device.u2f_info);

    device.ping().expect("ping");

    device.wink().expect("wink");

    let version = device.get_version().expect("version");

    println!("got u2f version: {:?}", version);

    let auth_response;

    loop {
        match device.authenticate(&challenge_param, &app_param, key_handle) {
            Ok(auth) => { auth_response = Some(auth); break; },
            Err(u2f::error::Error(u2f::error::ErrorKind::UserPresenceRequired, _)) => continue,
            Err(e) => Err(e).expect("authenticate")
        }

        std::thread::sleep(Duration::from_millis(200));
    }

    auth_response.unwrap()
}

pub fn write_auth_response_to_file(auth: &AuthenticateResponse) {
    let json = serde_json::to_string(&auth).expect("json");
    let path = path::Path::new("authresp.json");
    let mut f = fs::File::create(path).expect("open");
    let mut bytes = json.bytes().collect::<Vec<u8>>();
    f.write_all(&mut bytes[..]).expect("write");
}