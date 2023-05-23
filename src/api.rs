use crate::interface::server_api::IServerAPI;

struct ServerAPI;

impl IServerAPI for ServerAPI {
    fn register_user(&self, device_id: &String, app_version: &String, app_hash: &String) -> String {
        self.handle_register_user(device_id, app_version, app_hash)
    }
}

pub fn get_api_implementation() -> Box<dyn IServerAPI> {
    Box::new(ServerAPI{})
}

mod reg_newdevice;
