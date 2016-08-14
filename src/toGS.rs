use std::error::Error;

use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use std::process::{Command, Stdio};

use std::io::{Write,Read};
use nanomsg::{Socket, Protocol, Endpoint};

use appData::AppData;

pub struct ToGS{
    appData:    Weak<AppData>,
    toGS_socket:Mutex<Socket>,
    toGS_endpoint:Mutex<Endpoint>,
}

impl ToGS{
    pub fn new( appData:Arc<AppData> ) -> Result<ToGS,String> {
        let toGSFileName=format!("ipc:///tmp/ToGS_{}.ipc",appData.serverConfig.server_gamePort);

        let mut toGS_socket = match Socket::new(Protocol::Push){
            Ok(  s )=>s,
            Err( e )=>return Err(format!("Can not create toGS socket : {}",e.description())),
        };
        let mut toGS_endpoint = match toGS_socket.connect( &toGSFileName ){
            Ok(  s )=>s,
            Err( e )=>return Err(format!("Can not create toGS endpoint : {}",e.description())),
        };

        Ok(
            ToGS{
                appData:Arc::downgrade(&appData),
                toGS_socket:Mutex::new(toGS_socket),
                toGS_endpoint:Mutex::new(toGS_endpoint),
            }
        )
    }

    fn send(&self, msg:String ) {
        match self.toGS_socket.lock().unwrap().write( msg.as_bytes() ){
            Ok ( _ ) => {},
            Err( e ) => self.appData.upgrade().unwrap().log.print( format!("ToGS Write error : {}",e.description()) ),
        }
    }
}

impl Drop for ToGS{
    fn drop(&mut self) {
        self.toGS_endpoint.lock().unwrap().shutdown();
    }
}