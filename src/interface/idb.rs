pub trait IDatabaseCalls {
    fn is_connected(&self) -> bool;
    fn configure(&self) -> bool;
    fn check_apphash(&self, hash: &String) -> Option<bool>;
    fn check_appversion(&self, version: &String) -> Option<bool>;
    fn check_deviceid(&self, deviceid: &String) -> Option<bool>;
    fn register_new_device(&self, deviceid: &String, public_key: &String) -> Option<bool>;
}
