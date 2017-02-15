
error_chain! {
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
    }
}