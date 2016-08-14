use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use std::process::{Command, Stdio};

use std::io::{Write,Read};

use toGS::ToGS;
use fromGS::FromGS;
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
        self.endpoint.close();
        drop(self.socket);
    }
}


GameServer{
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


        //run

        let mut msg=String::new();
        fromGS.socket.set_receive_timeout(5000);

        match fromGS.socket.read_to_string(&mut msg){
            Ok( _ ) => {
                if msg.as_str()!="answer:FromGS is ready" {
                    return Err( format!("[ERROR]Can not run game server : FromGS: {}", msg) );
                }
            },
            Err( e ) => return Err( format!("[ERROR]Can not run game server : FromGS: {}", e.description()) ),
        }

        //===========================ToGS============================
        let toGSFileName=format!("ipc:///tmp/ToGS_{}.ipc",appData.serverConfig.server_gamePort);
        let mut toGS=try!(Channel::newPush( &fromGSFileName ));

        toGS.socket.set_send_timeout(200);
        match toGS.socket.write(b"answer:ToGS is ready"){
            Ok ( _ ) => {},
            Err( e ) => return Err( format!("[ERROR]Can not run game server : ToGS: {}", e.description()) ),
        }

        let mut msg=String::new();
        fromGS.socket.set_receive_timeout(1000);

        match fromGS.socket.read_to_string(&mut msg){
            Ok( _ ) => {
                if msg.as_str()!="answer:IPC is ready" {
                    return Err( format!("[ERROR]Can not run game server : IPC: {}", msg) );
                }
            },
            Err( e ) => return Err( format!("[ERROR]Can not run game server : IPC: {}", e.description()) ),
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



        let threadJoin=runThread( appData.clone(), gameServer.clone(), fromGS );

        Ok(())
    }

    fn runThread(
        appData:Arc<AppData>, //не даст другим потокам разрушить AppData
        gameServer:Arc<GameServer>, //не даст другим потокам разрушить GameServer
        mut fromGS:Channel
    ) -> ThreadJoin {
        thread::spawn(move || {
            fromGS.socket.set_receive_timeout(1500);

            while !{*gameServer.shouldClose.lock().unwrap()} {
                match fromGS.socket.read_to_string(&mut msg){
                    Ok( _ ) => {
                        if msg.as_str()=="close" {
                            break;
                        }
                    },
                    Err( e ) => {
                        match e {
                            io::ErrorKind::TimedOut =>
                                appData.log.print( format!("[ERROR]Connection with game server has been lost") ),
                            _=>
                                appData.log.print( format!("[ERROR]FromGS read error : {}", e.description()) ),
                        }

                        break;
                    },
                }

                let v: Vec<&str> = msg.splitn(2, ':').collect();

                if v.len()==2{
                    processFromGSCommand( v[0], v[1] );
                }else{
                    appData.log.print( format!("[ERROR]FromGS: \"{}\" is no command", msg.as_str()) );
                }
            }

            *gameServer.isRunning.lock().unwrap()=false;

            println!("ThreadEnd");

            //Выжидает, когда gameServer-ом никто не пользуется, и делает недоступным его использование
            *appData.gameServer.write().unwrap()=None;
            appData.log.print(format!("[INFO]Game server has been stoped"));
            //GameServer разрушается автоматически
        });
    }

    fn processFromGSCommand( appData:&Arc<AppData>, commandType:&str, args:&str ){
        match v[0] {
            //"answer" => *answer.lock().unwrap()=Some(String::from(v[1])),
            "online" => {},
            "print" => appData.log.print( String::from(v[1]) ),
            _=>appData.log.print( format!("[ERROR]FromGS: unknown command\"{}\"", v[0]) ),
        }
    }

    fn closeFromGS(&self){
        *self.shouldClose.lock().unwrap()=true;
        let mut fromGSTerminator_socket = Socket::new(Protocol::Push).unwrap();
        fromGSTerminator_socket.set_send_timeout(2000);
        let mut fromGSTerminator_endpoint = fromGSTerminator_socket.connect(&self.fromGSFileName).unwrap();
        fromGSTerminator_socket.write(b"close").unwrap();
    }

    pub fn send(&self, commandType:&str, msg:&str ) -> Result<(),String>{
        let msg=format!("{}:{}",commandType,msg);

        match {self.toGS.lock().unwrap().socket.write( msg.as_bytes() )} {
            Ok ( _ ) => Ok(()),
            Err( e ) => {
                let errorMessage=format!("[ERROR]ToGS Write error : {}",e.description());
                self.appData.upgrade().unwrap().log.print(errorMessage.clone());

                self.closeFromGS();

                Err( errorMessage );
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
    }
}
