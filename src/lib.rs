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

#[macro_use]
pub mod serde_enum;

pub mod api;
pub mod raw;
pub mod usb;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
