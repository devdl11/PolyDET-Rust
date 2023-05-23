use super::ServerAPI;
use crate::database::get_db_implementation;
use crate::requests::requests::{DATABASE_API_CONNECTION_ERROR, INVALID_APP_HASH, DEVICEID_REFUSED, OUTDATED_APP, API_UNKOWN_ERROR, APISignedDeviceResponse};
use crate::SEC_MODULE;

use crate::json;

impl ServerAPI {
    pub fn handle_register_user(&self, device_id: &String, app_version: &String, app_hash: &String) -> String {
        let hashres = get_db_implementation().check_apphash(app_hash);
        if hashres.is_none() {
            return json::to_string(DATABASE_API_CONNECTION_ERROR).unwrap();
        } else if !hashres.unwrap() {
            return json::to_string(INVALID_APP_HASH).unwrap();
        }

        let versionres = get_db_implementation().check_appversion(app_version);
        if versionres.is_none() {
            return json::to_string(DATABASE_API_CONNECTION_ERROR).unwrap();
        } else if !versionres.unwrap() {
            return json::to_string(OUTDATED_APP).unwrap();
        }

        let deviceres = get_db_implementation().check_deviceid(device_id);
        if deviceres.is_none() {
            return json::to_string(DATABASE_API_CONNECTION_ERROR).unwrap();
        } else if !deviceres.unwrap() {
            return json::to_string(DEVICEID_REFUSED).unwrap();
        }

        let lockedmodule = SEC_MODULE.lock();
        if lockedmodule.is_err() {
            return json::to_string(API_UNKOWN_ERROR).unwrap();
        }
        let lmodule_medium = lockedmodule.unwrap();
        let lmodule = lmodule_medium.as_ref().unwrap();
        let cert = lmodule.generate_cert_for_user(device_id, app_hash);
        if cert.is_none() {
            json::to_string(API_UNKOWN_ERROR).unwrap()
        } else {
            let cert = cert.unwrap();
            let isreg = get_db_implementation().register_new_device(device_id, &cert.public_key);
            if isreg.is_none() {
                json::to_string(DATABASE_API_CONNECTION_ERROR).unwrap()
            } else if !isreg.unwrap() {
                json::to_string(API_UNKOWN_ERROR).unwrap()
            } else {
                json::to_string(&APISignedDeviceResponse{
                    success: "OK",
                    certificate: cert.certificate,
                    private_key: cert.private_key
                }).unwrap()
            }
        }
    }
}