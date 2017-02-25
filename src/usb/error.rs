use hidapi;
use raw;
use usb::hid::U2fHidErrorCode;
use raw::frame::U2fStatusWord;

error_chain! {
    links {
        FramingError(raw::error::Error, raw::error::ErrorKind);
    }

    errors {
        RequestTooLong {
            description("request too long")
            display("request too long")
        }

        Hid(e: hidapi::HidError) {
            description("hid error")
            display("hid error: {}", e)
        }

        HidError(code: U2fHidErrorCode) {
            description("hid error")
            display("hid error: {:?}", code)
        }

        HidUnknownError(code: u8) {
            description("unknown hid error")
            display("unknown hid error: {}", code)
        }

        UnknownHidCommand(cmd: u8) {
            description("unknown hid command")
            display("unknown hid command: {}", cmd)
        }

        HidPacketTooSmall {
            description("hid packet is too small")
            display("hid packet is too small")
        }

        UnknownChannelId {
            description("unknown channel id")
            display("unknown channel id")
        }

        UnexpectedPacket {
            description("unexpected packet")
            display("unexpected packet")
        }

        InitResponseTooSmall {
            description("init response too small")
            display("init response too small")
        }

        ErrorStatus(status: U2fStatusWord) {
            description("error status")
            display("error status: {:?}", status)
        }

        UnknownErrorStatus(status: u16) {
            description("unknown error status")
            display("unknown error status: {}", status)
        }
    }
}