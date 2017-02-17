use usb;

error_chain! {
    links {
        Hid(usb::error::Error, usb::error::ErrorKind);
    }

    errors {
        ResponseFrameTooShort {
            description("response frame is too short")
            display("response frame is too short")
        }

        RequestDataTooLong {
            description("request data too long")
            display("request data too long")
        }

        ExpectedResponseDataTooLong {
            description("asked for too much response data")
            display("asked for too much response data")
        }

        ExpectedZeroResponseData {
            description("asked for zero response data")
            display("asked for zero response data")
        }

        FrameTooSmall {
            description("frame too small")
            display("frame too small")
        }

        ErrorStatus(status: u16) {
            description("error status")
            display("error status: {}", status)
        }

        UnrecognisedVersion {
            description("unrecognised version")
            display("unrecognised version")
        }
    }
}