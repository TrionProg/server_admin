use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use log::Log;
use serverConfig::ServerConfig;

use modManager::ModManager;
use webInterface::WebInterface;
use gameServer::GameServer;

use iron::Listening;

pub struct AppData{
    pub log:Log,
    pub serverConfig:ServerConfig,

    pub modManager:RwLock<Option< Arc<ModManager> >>,
    pub webInterface:RwLock<Option< Arc<WebInterface> >>,
    pub gameServer:RwLock<Option< Arc<GameServer> >>,
}

impl AppData{
    pub fn new( serverConfig:ServerConfig, log:Log ) -> AppData {
        AppData{
            log:log,
            serverConfig:serverConfig,

            modManager:RwLock::new(None),
            webInterface:RwLock::new(None),
            gameServer:RwLock::new(None),
        }
    }

    pub fn exit( &self ){
        /*
        match *self.gameServerChannel.read().unwrap() {
            Some( ref gsc ) => gsc.close(),
            None=>{},
        }
        */
        match *self.gameServer.read().unwrap(){
            Some( ref gs ) => gs.stop(),
            None=>{},
        }
        
        match *self.webInterface.read().unwrap(){
            Some( ref wi ) => wi.close(),
            None=>{},
        }

        *self.webInterface.write().unwrap()=None;
    }

    pub fn doModManager<T,F>(&self, f:F) -> T where F:FnOnce(&ModManager) -> T {
        match *self.modManager.read().unwrap(){
            Some( ref modManager) => {
                f( modManager )
            },
            None=>panic!("No mod manager"),
        }
    }

    pub fn doWebInterface<T,F>(&self, f:F) -> T where F:FnOnce(&WebInterface) -> T {
        match *self.webInterface.read().unwrap(){
            Some( ref webInterface) => {
                f( webInterface )
            },
            None=>panic!("No web interface"),
        }
    }

    /*
    pub fn doGameServerChannel<T,F>(&self, f:F) -> T where F:FnOnce(&GameServerChannel) -> T {
        match *self.gsChannel.read().unwrap(){
            Some( ref gsChannel) => {
                f( gsChannel )
            },
            None=>panic!("No gs channel"),
        }
    }
    */

    /*
    pub fn getModManager( &self ) -> &ModManager {
    match *self.modManager.read().unwrap(){
        Some( ref modManager) => {
        f(
                modManager
            },
            None=>panic!("No mod manager"),
        }
    }
    */
}
