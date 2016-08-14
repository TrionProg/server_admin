use std::error::Error;

use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use std::process::{Command, Stdio};

use std::io::{Write,Read};
use nanomsg::{Socket, Protocol, Endpoint};

use appData::AppData;

pub struct FromGS{
    isBlocked:Arc<Mutex<bool>>,
    answer:Arc<Mutex<Option<String>>>,
    threadHandle:thread::JoinHandle<()>,
    fromGSFileName:String,
}

impl FromGS{
    pub fn new( appData:Arc<AppData> ) -> Result<FromGS, String> {
        let fromGSFileName=format!("ipc:///tmp/FromGS_{}.ipc",appData.serverConfig.server_gamePort);

        let mut fromGS_socket = match Socket::new(Protocol::Pull){
            Ok(  s )=>s,
            Err( e )=>return Err( format!("Can not create fromGS socket : {}",e.description()) ),
        };

        fromGS_socket.set_receive_timeout(200);

        let mut fromGS_endpoint = match fromGS_socket.bind( &fromGSFileName ){
            Ok(  s )=>s,
            Err( e )=>return Err( format!("Can not create fromGS endpoint : {}",e.description()) ),
        };

        let isBlocked=Arc::new(Mutex::new(false));
        let answer=Arc::new(Mutex::new(None));

        let t_appData=appData.clone();
        let t_isBlocked=isBlocked.clone();
        let t_answer=answer.clone();

        let threadHandle=thread::spawn( move || FromGS::threadFunction(
            t_appData,
            fromGS_socket,
            fromGS_endpoint,
            t_isBlocked.clone(),
            t_answer.clone(),
        ));

        Ok(
            FromGS{
                isBlocked:isBlocked,
                answer:answer,
                threadHandle:threadHandle,
                fromGSFileName:fromGSFileName,
            }
        )
    }

    fn threadFunction (
        appData:Arc<AppData>,
        mut fromGS_socket:Socket,
        mut fromGS_endpoint:Endpoint,
        isBlocked:Arc<Mutex<bool>>,
        answer:Arc<Mutex<Option<String>>>,
    ){
        *isBlocked.lock().unwrap()=true;

        let mut msg=String::new();
        loop {
            match fromGS_socket.read_to_string(&mut msg){
                Ok( _ ) => {
                    match msg.find(':') {
                        Some( p ) => {
                            let v: Vec<&str> = msg.splitn(2, ':').collect();

                            if v.len()==2{
                                match v[0] {
                                    "close" => break,
                                    "answer" => *answer.lock().unwrap()=Some(String::from(v[1])),
                                    "print" => appData.log.print( String::from(v[1]) ),
                                    _=>appData.log.print(String::from("unknown")),
                                }
                            }else{
                                appData.log.print( format!("[ERROR]ToGS: \"{}\" is no command", msg.as_str()) );
                            }
                        },
                        None => appData.log.print( format!("[ERROR]FromGS: \"{}\" is no command", msg.as_str()) ),
                    }
                },
                Err( e ) => {
                    appData.log.print( format!("[ERROR]Can not read FromGS : {}",e.description()) );
                    break;
                },
            }

            msg.clear();
        }

        *isBlocked.lock().unwrap()=false;

        fromGS_endpoint.shutdown();
    }

    pub fn waitAnswer(&self, answer:&str) -> Result<(),String> {
        for i in 0..100 {
            thread::sleep_ms(50);

            let result={match *self.answer.lock().unwrap(){
                Some( ref a ) => {
                    if a==answer {
                        Some( Ok(()) )
                    }else{
                        Some( Err(format!("Answer \"{}\" does not equals to expected \"{}\"", a, answer)) )
                    }
                },
                None => None,
            }};

            match result{
                Some( res ) => {
                    *self.answer.lock().unwrap()=None;
                    return res;
                },
                None => {},
            }
        }

        return Err(format!("Timeout than answer \"{}\" has been expected",answer));
    }
}

impl Drop for FromGS{
    fn drop(&mut self) {
        if {*self.isBlocked.lock().unwrap()} {
            let mut fromGSTerminator_socket = Socket::new(Protocol::Push).unwrap();
            let mut fromGSTerminator_endpoint = fromGSTerminator_socket.connect(&self.fromGSFileName).unwrap();
            fromGSTerminator_socket.write(b"close:");
        }
        //*(self.threadHandle.clone()).join();
        //(*self.threadHandle).join();
    }
}
