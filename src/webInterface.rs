
use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use appData::AppData;

use iron::prelude::*;
use iron::status;
use router::Router;
use rustc_serialize::json;

use iron::error::HttpError;
use iron::Listening;
use std::io::Read;

pub struct WebInterface{
    pub appData:Weak<AppData>,
    pub listener:Mutex<Listening>,
    pub consoleIsActive:RwLock<bool>,
    pub consoleText:RwLock<String>,
}

#[derive(RustcEncodable, RustcDecodable)]
struct Greeting {
    msg: String
}

impl WebInterface{
    pub fn run( appData:Arc< AppData> ) -> Result<(), HttpError>  {

        let mut router = Router::new();

        router.get("/", hello_world);
        router.post("/set", set_greeting);

        fn hello_world(_: &mut Request) -> IronResult<Response> {
            let greeting = Greeting { msg: "Hello, World".to_string() };
            let payload = json::encode(&greeting).unwrap();
            Ok(Response::with((status::Ok, payload)))
        }

        // Receive a message by POST and play it back.
        fn set_greeting(request: &mut Request) -> IronResult<Response> {
            let mut payload = String::new();
            request.body.read_to_string(&mut payload);
            let request: Greeting = json::decode(payload.as_ref()).unwrap();
            let greeting = Greeting { msg: request.msg };
            let payload = json::encode(&greeting).unwrap();
            Ok(Response::with((status::Ok, payload)))
        }

        let address=format!("localhost:{}",appData.serverConfig.server_adminPort);
        let listener=try!(Iron::new(router).http(address.as_str()));

        let webInterface=Arc::new(WebInterface{
            appData:Arc::downgrade(&appData),
            listener:Mutex::new(listener),
            consoleIsActive:RwLock::new(false),
            consoleText:RwLock::new(String::with_capacity(256)),
        });

        *appData.webInterface.write().unwrap()=Some(webInterface.clone());
        *appData.log.webInterface.write().unwrap()=Some(webInterface);

        Ok( () )
    }

    pub fn close(&self){
        self.appData.upgrade().unwrap().log.print(format!("[INFO]Closing web interface"));

        *self.consoleIsActive.write().unwrap()=false;
        self.listener.lock().unwrap().close();

        self.appData.upgrade().unwrap().log.print(format!("[INFO]Web interface is closed"));
    }
}
