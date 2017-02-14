extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
pub mod serde_enum;

pub mod rp_u2f;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
