pub mod mysql_imp;

use crate::interface::idb::IDatabaseCalls;

// TODO: maybe use env. to retrieve the URL
// Yeah I know, It's a shity password, and what ?
pub const DATABASE_URI: &str = "mysql://root:polydetdb@localhost:3306/polydetdata";

#[cfg(database = "mysql")]
pub fn get_db_implementation() -> Box<dyn IDatabaseCalls> {
    use self::mysql_imp::MySQLDB;
    Box::new(MySQLDB{})
}

#[cfg(not(database = "mysql"))]
fn get_db_implementation() -> Box<dyn IDatabaseCalls> {
    // return another impl.
}

