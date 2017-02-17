use hidapi::*;
use self::hid::*;

pub mod error;
pub mod hid;

pub trait FidoExt {
    fn fido_devices(&self) -> Vec<HidDeviceInfo>;
}

impl FidoExt for HidApi {
    fn fido_devices(&self) -> Vec<HidDeviceInfo> {
        self.devices()
            .into_iter()
            .filter(|device| device.usage_page == FIDO_USAGE_PAGE && device.usage == U2F_USAGE)
            .collect::<Vec<HidDeviceInfo>>()
    }
}