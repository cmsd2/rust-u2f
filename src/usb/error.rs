use hidapi;

error_chain! {
    errors {
        RequestTooLong {
            description("request too long")
            display("request too long")
        }
        
        Hid(e: hidapi::HidError) {
            description("hid error")
            display("hid error: {}", e)
        }

        HidError {
            description("hid error")
            display("hid error")
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
    }
}