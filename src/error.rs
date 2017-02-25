
use usb;
use raw;
use webpki;

error_chain! {
    links {
        FramingError(raw::error::Error, raw::error::ErrorKind);
        HidError(usb::error::Error, usb::error::ErrorKind);
    }

    foreign_links {
    }

    errors {
        WebPkiError(e: webpki::Error) {
            description("webpki error")
            display("webpki error: {:?}", e)
        }

        UnrecognisedVersion {
            description("unrecognised version")
            display("unrecognised version")
        }

        InvalidChallengeParameter {
            description("invalid challenge parameter")
            display("invalid challenge parameter")
        }

        InvalidApplicationParameter {
            description("invalid application parameter")
            display("invalid application parameter")
        }

        KeyHandleTooLong {
            description("key handle too long")
            display("key handle too long")
        }

        InvalidRegistrationResponse {
            description("invalid registration response")
            display("invalid registration response")
        }

        UserPresenceRequired {
            description("user presence required")
            display("user presence required")
        }
    }
}