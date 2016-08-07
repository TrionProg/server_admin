
use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier};

use appData::AppData;

use iron::prelude::*;
use iron::status;
use router::Router;
use rustc_serialize::json;

use iron::error::HttpError;
use std::io::Read;

#[derive(RustcEncodable, RustcDecodable)]
struct Greeting {
    msg: String
}

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
    *appData.webInterfaceListener.lock().unwrap()=Some(listener);

    Ok( () )
}
