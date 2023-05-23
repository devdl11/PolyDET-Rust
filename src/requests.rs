pub mod requests {
    use rocket::serde::{Deserialize, Serialize};
    use sanitizer::prelude::*;


    #[derive(Deserialize, Sanitize)]
    #[serde(crate="rocket::serde")]
    pub struct RegisterNewDevice {
        #[sanitize(trim, lower_case, alphanumeric)]
        pub app_hash: String,
        #[sanitize(trim, lower_case, alphanumeric)]
        pub app_version: String,
        #[sanitize(trim, lower_case, alphanumeric)]
        pub device_id: String,
    }

    #[derive(Serialize)]
    #[serde(crate="rocket::serde")]
    pub struct APIError {
        pub error: &'static str
    }
    pub struct APISuccess {
        pub success: &'static str   
    }

    #[derive(Serialize)]
    #[serde(crate="rocket::serde")]
    pub struct APISignedDeviceResponse {
        pub success: &'static str,
        pub certificate: String,
        pub private_key: String
    }

    pub const ENDPOINT_API_ERROR : &APIError = &APIError {
        error: "Invalid Path or Data"
    };

    pub const DATABASE_API_CONNECTION_ERROR : &APIError = &APIError {
        error : "Database Connection Error"
    };

    pub const INVALID_APP_HASH : &APIError = &APIError {
        error : "Invalid App Hash"
    };

    pub const OUTDATED_APP : &APIError = &APIError {
        error : "Outdated App"
    };

    pub const DEVICEID_REFUSED : &APIError = &APIError {
        error : "Device ID Uneligible"
    };

    pub const API_UNKOWN_ERROR : &APIError = &APIError {
        error : "Something Bad Happend"
    };

    pub const ENDPOINT_API_SUCCESS : &APISuccess = &APISuccess {
        success: "Operation succeed"
    };

}
