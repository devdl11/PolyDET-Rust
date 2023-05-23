use crate::{interface::idb::IDatabaseCalls};

use mysql::prelude::*;
use mysql::*;
use version_compare::Cmp;

use self::{scripts::{GET_APPHASH_SETTING, GET_MIN_APPVERSION_SETTING, GET_REGISTEREDDEVICE_BY_DEVICEID, DROP_SECURESTORAGEKEY_BY_PUBLIC_KEY, DROP_REGISTEREDDEVICE_BY_DEVICEID, INSERT_NEW_REGISTEREDDEVICE}, tables::RegisteredDevice};

use super::DATABASE_URI;

pub struct MySQLDB {}

impl MySQLDB {
    pub fn get_connection(&self) -> Option<PooledConn> {
        let pool = Pool::new(DATABASE_URI);
        if pool.is_err() {
            eprintln!("Cannot connect to database !");
            return None;
        }
        let conn = pool.unwrap().get_conn();
        if conn.is_err() {
            eprintln!("Opa !");
            return None;
        }
        Some(conn.unwrap())
    }
}

impl IDatabaseCalls for MySQLDB {
    fn is_connected(&self) -> bool {
        let pool = Pool::new(DATABASE_URI);
        match pool {
            Ok(_) => true,
            Err(_) => false
        }
    }

    fn configure(&self) -> bool {
        let mut conn = self.get_connection();
        if conn.is_none() {
            return false;
        }
        
        for query in [
            scripts::SECURESTORAGEKEYS_CREATETABLE,
            scripts::GLOBALSETTINGS_CREATETABLE,
            scripts::INWAITINGEVOLUTION_CREATETABLE,
            scripts::PLANNINGEVOLUTION_CREATETABLE,
            scripts::REGISTEREDDEVICE_CREATETABLE,
        ] {
            let result = conn.as_mut().unwrap().query_drop(query);
            if result.is_err() {
                eprintln!("{}", result.err().unwrap());
                return false;
            }
        }

    println!("DB is ready !");
    true
    }

    fn check_apphash(&self, hash: &String) -> Option<bool> {
        let conn = self.get_connection();
        if conn.is_none() {
            return None;
        }
        let hashvalue: std::result::Result<Vec<String>, Error> = conn.unwrap().query(GET_APPHASH_SETTING);
        if hashvalue.is_err() {
            return None;
        }
        let hashvalue = hashvalue.unwrap();
        if hashvalue.is_empty() {
            return Some(true);
        }
        for hashter in hashvalue.iter() {
            if *hashter == *hash {
                return Some(true);
            }
        }
        Some(false)
    }

    fn check_appversion(&self, version: &String) -> Option<bool> {
        use version_compare::Version;
        let conn = self.get_connection();
        if conn.is_none() {
            return None;
        }
        let user_version = Version::from(version);
        if user_version.is_none() {
            return Some(false);
        }
        let user_version = user_version.unwrap();

        let minappversion: std::result::Result<Option<String>, Error> = conn.unwrap().query_first(GET_MIN_APPVERSION_SETTING);
        if minappversion.is_err() {
            return None;
        }
        let minappversion = minappversion.unwrap();
        if minappversion.is_none() {
            return Some(true);
        }
        let minappversion = minappversion.unwrap();
        
        let system_version = Version::from(&minappversion);
        if system_version.is_none() {
            return Some(true);
        }

        let system_version = system_version.unwrap();
        if user_version.compare(system_version) == Cmp::Ge {
            Some(true)
        } else {
            Some(false)
        }
    }

    fn check_deviceid(&self, deviceid: &String) -> Option<bool> {
        let conn = self.get_connection();
        if conn.is_none() {
            return None;
        }
        let mut conn = conn.unwrap();
        let stmt = conn.prep(GET_REGISTEREDDEVICE_BY_DEVICEID);
        if stmt.is_err() {
            return None;
        }
        let stmt = stmt.unwrap();
        let registereddevice : Result<Option<RegisteredDevice>, Error> = conn.exec_first(stmt, params! {"uniqid" => deviceid});
        if registereddevice.is_err() {
            eprintln!("{}", registereddevice.err().unwrap());
            return None;
        }
        let registereddevice = registereddevice.unwrap();
        if registereddevice.is_none() {
            return Some(true);
        }
        let registereddevice = registereddevice.unwrap();
        // We clean up all older data associated with this deviceid.
        let stmt = conn.prep(DROP_SECURESTORAGEKEY_BY_PUBLIC_KEY);
        if stmt.is_err() {
            return None;
        }
        let stmt = stmt.unwrap();
        let simplexec = conn.exec_drop(stmt, params! {"pubkey" => registereddevice.publickey});
        if simplexec.is_err() {
            eprintln!("{}", simplexec.err().unwrap());
            return None;
        }

        let stmt = conn.prep(DROP_REGISTEREDDEVICE_BY_DEVICEID);
        if stmt.is_err() {
            return None;
        }
        let stmt = stmt.unwrap();
        let simplexec = conn.exec_drop(stmt, params! {"uniqid" => registereddevice.uniqueid});

        if simplexec.is_err() {
            eprintln!("{}", simplexec.err().unwrap());
            return None;
        }

        Some(true)
    }

    fn register_new_device(&self, deviceid: &String, public_key: &String) -> Option<bool> {
        let conn = self.get_connection();
        if conn.is_none() {
            return None;
        }
        let mut conn = conn.unwrap();
        let stmt = conn.prep(INSERT_NEW_REGISTEREDDEVICE);
        if stmt.is_err() {
            return None;
        }
        let stmt = stmt.unwrap();
        let simplexec = conn.exec_drop(stmt, params! {"uniqid" => deviceid, "pubkey" => public_key});
        if simplexec.is_err() {
            eprintln!("{}", simplexec.err().unwrap());
            None
        } else {
            Some(true)
        }
    }
}

mod tables {
    use mysql_common::{prelude::FromRow, time::PrimitiveDateTime};

    // Tables Definition
    #[derive(PartialEq, Eq, FromRow)]
    pub struct InWaitingEvolution {
        pub id: i32,
        pub date: PrimitiveDateTime,
        pub bywho: String,
        pub data: Vec<u8>,
    }

    #[derive(PartialEq, Eq, FromRow)]
    pub struct SecureStorageKeys {
        pub id: i32,
        pub public_key: String,
        pub cipher_key: String,
    }

    #[derive(PartialEq, Eq, FromRow)]
    pub struct PlanningEvolution {
        pub id: i32,
        pub date: PrimitiveDateTime,
        pub data_bundle: Vec<u8>,
    }

    #[derive(PartialEq, Eq, FromRow)]
    pub struct GlobalSettings {
        pub id: i32,
        pub parameter: String,
        pub value: String,
    }

    #[derive(PartialEq, Eq, FromRow)]
    pub struct RegisteredDevice {
        pub id: i32,
        pub uniqueid: String,
        pub publickey: String,
    }

}

mod scripts {
    pub const SECURESTORAGEKEYS_CREATETABLE: &'static str = r"CREATE TABLE IF NOT EXISTS `polydetdata`.`SecureStorageKeys` (
`id` INT NOT NULL AUTO_INCREMENT,
`PublicKey` VARCHAR(128) NOT NULL,
`CipherKey` VARCHAR(128) NOT NULL,
PRIMARY KEY (`id`),
UNIQUE INDEX `id_UNIQUE` (`id` ASC) VISIBLE,
UNIQUE INDEX `PublicKey_UNIQUE` (`PublicKey` ASC) VISIBLE);
";
    pub const PLANNINGEVOLUTION_CREATETABLE: &'static str = r"CREATE TABLE IF NOT EXISTS `polydetdata`.`PlanningEvolution` (
`id` INT NOT NULL AUTO_INCREMENT,
`date` DATETIME NOT NULL,
`data_bundle` BLOB NOT NULL,
PRIMARY KEY (`id`),
UNIQUE INDEX `id_UNIQUE` (`id` ASC) VISIBLE,
UNIQUE INDEX `date_UNIQUE` (`date` ASC) VISIBLE);
";
    pub const INWAITINGEVOLUTION_CREATETABLE: &'static str = r"CREATE TABLE IF NOT EXISTS `polydetdata`.`InWaitingEvolution` (
`id` INT NOT NULL AUTO_INCREMENT,
`date` DATETIME NOT NULL,
`bywho` VARCHAR(128) NOT NULL,
`data` BLOB NOT NULL,
PRIMARY KEY (`id`),
UNIQUE INDEX `id_UNIQUE` (`id` ASC) VISIBLE,
UNIQUE INDEX `date_UNIQUE` (`date` ASC) VISIBLE);
";
    pub const GLOBALSETTINGS_CREATETABLE: &'static str = r"CREATE TABLE IF NOT EXISTS `polydetdata`.`GlobalSettings` (
`id` INT NOT NULL AUTO_INCREMENT,
`parameter` VARCHAR(32) NOT NULL,
`value` VARCHAR(128) NULL,
PRIMARY KEY (`id`),
UNIQUE INDEX `id_UNIQUE` (`id` ASC) VISIBLE,
UNIQUE INDEX `parameter_UNIQUE` (`parameter` ASC) VISIBLE);
";
    pub const REGISTEREDDEVICE_CREATETABLE: &'static str = r"CREATE TABLE IF NOT EXISTS `polydetdata`.`RegisteredDevice` (
`id` INT NOT NULL AUTO_INCREMENT,
`uniqueid` VARCHAR(128) NOT NULL,
`publickey` VARCHAR(128) NOT NULL,
PRIMARY KEY (`id`),
UNIQUE INDEX `id_UNIQUE` (`id` ASC) VISIBLE,
UNIQUE INDEX `uniqueid_UNIQUE` (`uniqueid` ASC) VISIBLE,
UNIQUE INDEX `publickey_UNIQUE` (`publickey` ASC) VISIBLE);
";

    pub const GET_APPHASH_SETTING : &'static str = r"SELECT * FROM GlobalSettings WHERE parameter='apphash'";
    pub const GET_MIN_APPVERSION_SETTING : &'static str = r"SELECT * FROM GlobalSettings WHERE parameter='minappversion'";
    pub const GET_REGISTEREDDEVICE_BY_DEVICEID : &'static str = r"SELECT * FROM RegisteredDevice WHERE uniqueid=:uniqid";
    pub const DROP_REGISTEREDDEVICE_BY_DEVICEID : &'static str = r"DELETE FROM RegisteredDevice WHERE uniqueid=:uniqid";
    pub const DROP_SECURESTORAGEKEY_BY_PUBLIC_KEY : &'static str = r"DELETE FROM SecureStorageKeys WHERE PublicKey=:pubkey";
    pub const INSERT_NEW_REGISTEREDDEVICE : &'static str = r"INSERT INTO RegisteredDevice (uniqueid, publickey) VALUES (:uniqid, :pubkey)";

}
