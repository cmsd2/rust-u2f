
#[derive(Serialize,Deserialize,Copy,Clone,Debug,PartialEq)]
pub enum U2fRequestType {
    #[serde(rename="u2f_register_request")]
    RegisterRequest,

    #[serde(rename="u2f_sign_request")]
    SignRequest,
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq)]
pub struct U2fRequest {
    #[serde(rename="type")]
    pub request_type: U2fRequestType,

    #[serde(rename="appId")]
    pub app_id: Option<String>,

    #[serde(rename="timeoutSeconds")]
    pub timeout_seconds: Option<u32>,
    
    #[serde(rename="requestId")]
    pub request_id: Option<u32>,
}

#[derive(Serialize,Deserialize,Copy,Clone,Debug,PartialEq)]
pub enum U2fResponseType {
    #[serde(rename="u2f_register_response")]
    RegisterResponse,

    #[serde(rename="u2f_sign_response")]
    SignResponse,
}

enum_number!(ErrorCode {
    Ok = 0,
    OtherError = 1,
    BadRequest = 2,
    ConfigurationUnsupported = 3,
    DeviceIneligible = 4,
    Timeout = 5,
});

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq)]
#[serde(untagged)]
pub enum U2fResponseData {
    Error {
        #[serde(rename="errorCode")]
        error_code: ErrorCode,

        #[serde(rename="errorMessage")]
        error_message: Option<String>,
    },

    RegisterResponse {
        version: String,

        #[serde(rename="registrationData")]
        registration_data: String,

        #[serde(rename="clientData")]
        client_data: String,
    },

    SignResponse {
        #[serde(rename="keyHandle")]
        key_handle: String,

        #[serde(rename="signatureData")]
        signature_data: String,

        #[serde(rename="clientData")]
        client_data: String,
    }
}

#[derive(Serialize,Deserialize,Clone,Debug,PartialEq)]
pub struct U2fResponse {
    #[serde(rename="type")]
    pub response_type: U2fResponseType,

    #[serde(rename="responseData")]
    pub response_data: U2fResponseData,

    #[serde(rename="requestId")]
    pub request_id: Option<u32>,
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;

    #[test]
    fn test_request_serialization() {
        let reg = U2fRequest { request_type: U2fRequestType::RegisterRequest, app_id: None, timeout_seconds: None, request_id: None };
        assert_eq!(serde_json::to_string(&reg).unwrap(), "{\"type\":\"u2f_register_request\",\"appId\":null,\"timeoutSeconds\":null,\"requestId\":null}");
    }

    #[test]
    fn test_registration_response_deserialization() {
        let err_response_json = "{\"type\":\"u2f_register_response\",\"responseData\":{\"version\":\"0\",\"registrationData\":\"foo\",\"clientData\":\"bar\"},\"requestId\":null}";
        let err_response = serde_json::from_str::<U2fResponse>(err_response_json).unwrap();
        assert_eq!(err_response, U2fResponse {
            response_type: U2fResponseType::RegisterResponse,
            response_data: U2fResponseData::RegisterResponse {
                version: "0".to_string(),
                registration_data: "foo".to_string(),
                client_data: "bar".to_string(),
            },
            request_id: None,
        });
    }

    #[test]
    fn test_sign_response_deserialization() {
        let err_response_json = "{\"type\":\"u2f_sign_response\",\"responseData\":{\"keyHandle\":\"foo\",\"signatureData\":\"bar\",\"clientData\":\"baz\"},\"requestId\":null}";
        let err_response = serde_json::from_str::<U2fResponse>(err_response_json).unwrap();
        assert_eq!(err_response, U2fResponse {
            response_type: U2fResponseType::SignResponse,
            response_data: U2fResponseData::SignResponse {
                key_handle: "foo".to_string(),
                signature_data: "bar".to_string(),
                client_data: "baz".to_string(),
            },
            request_id: None,
        });
    }

    #[test]
    fn test_error_response_deserialization() {
        let err_response_json = "{\"type\":\"u2f_register_response\",\"responseData\":{\"errorCode\":0,\"errorMessage\":null},\"requestId\":null}";
        let err_response = serde_json::from_str::<U2fResponse>(err_response_json).unwrap();
        assert_eq!(err_response, U2fResponse {
            response_type: U2fResponseType::RegisterResponse,
            response_data: U2fResponseData::Error {
                error_code: ErrorCode::Ok,
                error_message: None,
            },
            request_id: None,
        });
    }
}