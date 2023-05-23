pub struct UserCertificate {
    pub device_id: String,
    pub app_hash: String,
    pub public_key: String,
    pub certificate: String,
    pub private_key: String,
}

pub trait IServerAPI {
    fn register_user(&self, device_id: &String, app_version: &String, app_hash: &String) -> String;

}
