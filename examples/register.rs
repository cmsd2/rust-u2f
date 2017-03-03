extern crate u2f;
extern crate hidapi;
extern crate serde_json;

use u2f::usb::hid::*;
use u2f::usb::*;
use u2f::*;
use hidapi::*;
use std::fs;
use std::path;
use std::io::Write;
use std::time::Duration;

pub fn main() {

    let api = HidApi::new().unwrap();

    let devices = api.fido_devices();
    
    for device in devices.iter() {
        println!("{:#?}", device);
    }

    if let Some(ref device) = devices.first() {
        let challenge = vec![0;32];
        let app_param = vec![0;32];

        if let Some(response) = register(&api, device, &challenge, &app_param) {
            write_response_to_file(&response, path::Path::new("regresp.json"));
        }
    } else {
        println!("no fido device found");
        std::process::exit(1);
    }
}

pub fn register(api: &HidApi, device_info: &HidDeviceInfo, challenge_param: &[u8], app_param: &[u8]) -> Option<RegisterResponse> {

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
    
    let reg_response;

    loop {
        match device.register(challenge_param, app_param) {
            Ok(reg) => { reg_response = Some(reg); break; },
            Err(u2f::error::Error(u2f::error::ErrorKind::UserPresenceRequired, _)) => continue,
            Err(e) => Err(e).expect("register")
        }

        std::thread::sleep(Duration::from_millis(200));
    }

    reg_response
}

pub fn write_response_to_file(reg_response: &RegisterResponse, p: &path::Path) {
    let json = serde_json::to_string(&reg_response).expect("json");
    let mut f = fs::File::create(p).expect("open");
    let mut bytes = json.bytes().collect::<Vec<u8>>();
    f.write_all(&mut bytes[..]).expect("write");
}