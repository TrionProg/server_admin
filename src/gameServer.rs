use std::error::Error;

use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use std::process::{Command, Stdio};

use std::io::{Write,Read, ErrorKind};
use nanomsg::{Socket, Protocol, Endpoint};

use appData::AppData;

struct Channel{
    socket:Socket,
    endpoint:Endpoint,
}

impl Channel{
    pub fn newPull( fileName:&str ) -> Result<Channel, String>{
        let mut socket = match Socket::new(Protocol::Pull){
            Ok(  s )=>s,
            Err( e )=>return Err( format!("Can not create socket \"{}\" : {}", fileName, e.description()) ),
        };

        socket.set_receive_timeout(5000);

        let mut endpoint = match socket.bind( fileName ){
            Ok(  s )=>s,
            Err( e )=>return Err( format!("Can not create endpoint \"{}\" : {}", fileName, e.description()) ),
        };

        Ok(
            Channel{
                socket:socket,
                endpoint:endpoint,
            }
        )
    }

    pub fn newPush( fileName:&str ) -> Result<Channel, String> {
        let mut socket = match Socket::new(Protocol::Push){
            Ok(  s )=>s,
            Err( e )=>return Err(format!("Can not create socket \"{}\" : {}", fileName, e.description())),
        };

        socket.set_send_timeout(200);

        let mut endpoint = match socket.connect( fileName ){
            Ok(  s )=>s,
            Err( e )=>return Err(format!("Can not create endpoint \"{}\" : {}", fileName, e.description())),
        };

        Ok(
            Channel{
                socket:socket,
                endpoint:endpoint,
            }
        )
    }
}

impl Drop for Channel{
    fn drop(&mut self){
        self.endpoint.shutdown();
    }
}


pub struct GameServer{
    appData:        Weak<AppData>,
    toGS:           Mutex<Channel>,
    fromGSFileName: String,
    shouldClose:    Mutex<bool>,
    isRunning:      Mutex<bool>,
}

impl GameServer {
    pub fn run( appData:Arc<AppData> ) -> Result<(), String> {
        //===========================FromGS===========================
        let fromGSFileName=format!("ipc:///tmp/FromGS_{}.ipc",appData.serverConfig.server_gamePort);
        let mut fromGS=try!(Channel::newPull( &fromGSFileName ));


        match Command::new("./server_game").stdout(Stdio::null()).spawn(){
            Ok ( _ ) => {},
            Err( e ) => return Err( format!("Can not execute server_game : {}",e.description()) ),
        }

        let mut msg=String::new();
        fromGS.socket.set_receive_timeout(5000);

        match fromGS.socket.read_to_string(&mut msg){
            Ok( _ ) => {
                if msg.as_str()!="answer:FromGS is ready" {
                    return Err( format!("Can not open FromGS: {}", msg) );
                }
            },
            Err( e ) => return Err( format!("Can not open FromGS: {}", e.description()) ),
        }

        //===========================ToGS============================
        let toGSFileName=format!("ipc:///tmp/ToGS_{}.ipc",appData.serverConfig.server_gamePort);
        let mut toGS=try!(Channel::newPush( &toGSFileName ));

        toGS.socket.set_send_timeout(200);
        match toGS.socket.write(b"answer:ToGS is ready"){
            Ok ( _ ) => {},
            Err( e ) => return Err( format!("Can not open ToGS: {}", e.description()) ),
        }

        let mut msg=String::new();
        fromGS.socket.set_receive_timeout(2000);

        match fromGS.socket.read_to_string(&mut msg){
            Ok( _ ) => {
                if msg.as_str()!="answer:IPC is ready" {
                    return Err( format!("Can not create IPC: {}", msg) );
                }
            },
            Err( e ) => return Err( format!("Can not create IPC: {}", e.description()) ),
        }

        //===========================GameServer======================

        let gameServer=Arc::new(
            GameServer{
                appData:Arc::downgrade(&appData),
                toGS:Mutex::new(toGS),
                fromGSFileName:fromGSFileName,
                shouldClose:Mutex::new(false),
                isRunning:Mutex::new(true),
            }
        );

        *appData.gameServer.write().unwrap()=Some(gameServer.clone());

        GameServer::runThread( appData.clone(), gameServer.clone(), fromGS );

        Ok(())
    }

    fn runThread(
        appData:Arc<AppData>, //не даст другим потокам разрушить AppData
        gameServer:Arc<GameServer>, //не даст другим потокам разрушить GameServer
        mut fromGS:Channel
    ) {
        thread::spawn(move || {
            let mut msg=String::with_capacity(1024);
            fromGS.socket.set_receive_timeout(1500);

            while !{*gameServer.shouldClose.lock().unwrap()} {
                match fromGS.socket.read_to_string(&mut msg){
                    Ok( _ ) => {
                        if msg.as_str()=="close:" {
                            break;
                        }

                        let v: Vec<&str> = msg.splitn(2, ':').collect();

                        if v.len()==2{
                            GameServer::processFromGSCommand( &appData, v[0], v[1] );
                        }else{
                            appData.log.print( format!("[ERROR]FromGS: \"{}\" is no command", msg.as_str()) );
                        }
                    },
                    Err( e ) => {
                        match e.kind() {
                            ErrorKind::TimedOut =>
                                appData.log.print( format!("[ERROR]Connection with game server has been lost") ),
                            _=>
                                appData.log.print( format!("[ERROR]FromGS read error : {}", e.description()) ),
                        }

                        break;
                    },
                }
                
                msg.clear();
            }

            *gameServer.isRunning.lock().unwrap()=false;

            //Выжидает, когда gameServer-ом никто не пользуется, и делает недоступным его использование
            *appData.gameServer.write().unwrap()=None;
            appData.log.print(format!("[INFO]Game server connection has been closed"));
            //GameServer разрушается автоматически
        });
    }

    fn processFromGSCommand( appData:&Arc<AppData>, commandType:&str, args:&str ){
        match commandType {
            //"answer" => *answer.lock().unwrap()=Some(String::from(v[1])),
            "online" => {},
            "print" => appData.log.print( String::from(args) ),
            _=>appData.log.print( format!("[ERROR]FromGS: unknown command\"{}\"", args) ),
        }
    }

    fn close(&self){
        *self.shouldClose.lock().unwrap()=true;
        let mut fromGSTerminator_socket = Socket::new(Protocol::Push).unwrap();
        fromGSTerminator_socket.set_send_timeout(2000);
        let mut fromGSTerminator_endpoint = fromGSTerminator_socket.connect(&self.fromGSFileName).unwrap();
        fromGSTerminator_socket.write(b"close:").unwrap();

        while {*self.isRunning.lock().unwrap()} {
            thread::sleep_ms(100);
        }
    }

    pub fn send(&self, commandType:&str, msg:&str ) -> Result<(),String>{
        let msg=format!("{}:{}",commandType,msg);

        match {self.toGS.lock().unwrap().socket.write( msg.as_bytes() )} {
            Ok ( _ ) => Ok(()),
            Err( e ) => {
                let errorMessage=format!("[ERROR]ToGS Write error : {}",e.description());
                self.appData.upgrade().unwrap().log.print(errorMessage.clone());

                self.close();

                Err( errorMessage )
            },
        }
    }

    pub fn stop(&self){
        let appData=self.appData.upgrade().unwrap();

        appData.log.print(format!("[INFO]Stopping game server"));
        self.send("cmd", "stop");

        while {*self.isRunning.lock().unwrap()} {
            thread::sleep_ms(100);
        }

        appData.log.print(format!("[INFO]Game server has been stoped"));
    }
}


/*
use std::error::Error;

use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use std::process::{Command, Stdio};

use std::io::{Write,Read};
use nanomsg::{Socket, Protocol, Endpoint};

use appData::AppData;

pub struct GameServer{
    appData:    Weak<AppData>,
    toGS:       ToGS,
    fromGS:     FromGS,
}

impl GameServer{
    pub fn run( appData:Arc<AppData> ) -> Result<(),String> {
        //appData.log.print(format!("Running game server"))
        let fromGS=try!(FromGS::new( appData.clone() ));

        //execute

        appData.log.print(format!("Waiting"));

        try!(fromGS.waitAnswer("FromGS is opened"));

        let toGS=try!(ToGS::new( appData.clone() ));

        try!(toGS.send("answer","ToGS is opened"));

        try!(fromGS.waitAnswer("IPC is ready"));

        println!("yeah");

        thread::sleep_ms( 15000 );

        //toGS.send("close","");

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
*/
