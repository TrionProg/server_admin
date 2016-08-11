use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use log::Log;
use serverConfig::ServerConfig;

use modManager::ModManager;
use webInterface::WebInterface;

use iron::Listening;

pub struct AppData{
    pub log:Log,
    pub serverConfig:ServerConfig,

    pub modManager:RwLock<Option< Arc<ModManager> >>,
    pub webInterface:RwLock<Option< Arc<WebInterface> >>,
}

impl AppData{
    pub fn new( serverConfig:ServerConfig, log:Log ) -> AppData {
        AppData{
            log:log,
            serverConfig:serverConfig,

            modManager:RwLock::new(None),
            webInterface:RwLock::new(None),
        }
    }

    pub fn exit( &self ){
        match *self.webInterface.read().unwrap(){
            Some( ref wi ) => wi.close(),
            None=>{},
        }
    }

    /*
    pub fn getModManager( &self ) -> &ModManager {
        match *self.modManager.read().unwrap(){
            Some( ref modManager) => {
                modManager
            },
            None=>panic!("No mod manager"),
        }
    }
    */
}
