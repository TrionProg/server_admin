use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier};

use serverConfig::ServerConfig;
use iron::Listening;

pub struct AppData{
    pub serverConfig:ServerConfig,
    pub webInterfaceListener:Mutex<Option< Listening >>,
}

impl AppData{
    pub fn new( serverConfig:ServerConfig ) -> AppData {
        AppData{
            serverConfig:serverConfig,
            webInterfaceListener:Mutex::new(None),
        }
    }
}
