mod interface;

mod security;
use security::security::SecurityModule;

mod database;
use database::get_db_implementation;

mod requests;
use requests::requests::{RegisterNewDevice, ENDPOINT_API_ERROR};

mod api;
use api::get_api_implementation;

use std::{process::exit, sync::Mutex};
use rocket::serde::json::{Json, self};

#[macro_use]
extern crate rocket;


const VERSION: &str = env!("CARGO_PKG_VERSION");

const CA_PATH: &str = "ca_cert.pem";
const PK_PATH: &str = "pk.key";

static SEC_MODULE: Mutex<Option<SecurityModule>> = Mutex::new(None);

#[get("/hello")]
fn hello() -> &'static str {
    "Hey !"
}

#[get("/dbconnectivity")]
fn db_is_online() -> &'static str {
    if get_db_implementation().is_connected() {
        "Online"
    } else {
        "Offline"
    }
}

#[post("/registernewdevice", format="application/json", data="<login>")]
fn register_device(login:Json<RegisterNewDevice>) -> String {
    get_api_implementation().register_user(&login.device_id, &login.app_version, &login.app_hash)   
}

#[catch(404)]
fn endpoint_api_404() -> String {
    json::to_string(ENDPOINT_API_ERROR).unwrap()
}

#[catch(422)]
fn endpoint_api_422() -> String {
    endpoint_api_404()
}

#[launch]
fn rocket() -> _ {
    println!("Running Rust Server {VERSION}");
    if !get_db_implementation().configure() {
        exit(1)
    }

    *SEC_MODULE.lock().unwrap() = SecurityModule::new(CA_PATH, PK_PATH);
    if SEC_MODULE.lock().unwrap().is_none() {
        println!("No ca_cert, let's generete it !");
        let success: bool = SecurityModule::init_env(CA_PATH, PK_PATH);
        if success {
            println!("We have a CA !");
        } else {
            eprintln!("Error : cannot generate CA !");
            exit(1);
        }
        *SEC_MODULE.lock().unwrap() = SecurityModule::new(CA_PATH, PK_PATH);
    }

    rocket::build().mount("/api", routes![
        hello, 
        db_is_online, 
        register_device,
        ])
        .register("/api", catchers![
            endpoint_api_404,
            endpoint_api_422,
        ])
}
