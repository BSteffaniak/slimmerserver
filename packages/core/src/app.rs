use std::{
    fmt,
    sync::{Arc, Mutex},
};

use sqlite::Connection;

pub struct AppState {
    pub service_port: u16,
    pub db: Option<Db>,
}

#[derive(Clone)]
pub struct Db {
    pub library: Arc<Mutex<Connection>>,
}

impl fmt::Debug for Db {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Db")
            .field("library", &"{{db connection}}")
            .finish()
    }
}
