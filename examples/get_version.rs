extern crate u2f;
extern crate hidapi;
extern crate serde_json;

use u2f::usb::hid::*;
use u2f::usb::*;
use u2f::*;
use hidapi::*;

pub fn main() {
    let api = HidApi::new().unwrap();

    let devices = api.fido_devices();
    
    for device in devices.iter() {
        println!("{:#?}", device);
    }

    if let Some(ref device) = devices.first() {
        get_version(&api, device);
    } else {
        println!("no fido device found");
        std::process::exit(1);
    }
}

pub fn get_version(api: &HidApi, device_info: &HidDeviceInfo) {

    let hid_device = 
        //api.open(device.vendor_id, device.product_id).expect("open");
        api.open_path(&device_info.path).expect("open");

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

    println!("got u2f version: {:?} for device vendor_id={} product_id={} path={}", 
        version, device_info.vendor_id, device_info.product_id, device_info.path);
}