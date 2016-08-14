use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use std::process::{Command, Stdio};

use std::io::{Write,Read};

use toGS::ToGS;
use fromGS::FromGS;
use appData::AppData;

pub struct GameServer{
    appData:    Weak<AppData>,
    toGS:ToGS,
    fromGS:FromGS,
}

impl GameServer{
    pub fn run( appData:Arc<AppData> ) -> Result<(),String> {
        //appData.log.print(format!("Running game server"))
        let fromGS=try!(FromGS::new( appData.clone() ));

        //execute

        appData.log.print(format!("Waiting"));

        //thread::sleep_ms( 10000 );

        try!(fromGS.waitAnswer(":FromGS is opened"));

        appData.log.print(format!("Opened"));


        let toGS=try!(ToGS::new( appData.clone() ));

        appData.log.print(format!("toGS"));

        /*
        try!(toGS.send("ToGS is opened"));

        try!(fromGS.waitAnswer(":IPC is ready"));
        */

        thread::sleep_ms( 1000 );

        let gameServer=Arc::new(
            GameServer{
                appData:Arc::downgrade(&appData),
                fromGS:fromGS,
                toGS:toGS,
            }
        );

        *appData.gameServer.write().unwrap()=Some(gameServer);

        Ok(())
    }
}
