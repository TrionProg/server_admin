#![allow(non_snake_case)]

extern crate rustc_serialize;
extern crate iron;
extern crate router;
extern crate curl;
extern crate zip;

mod log;
mod config;
mod serverConfig;
mod appData;
mod version;
mod modManager;

mod consoleInterface;
mod webInterface;
mod commandProcessor;



use std::error::Error;

use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier};

use log::Log;
use serverConfig::ServerConfig;
use appData::AppData;
use modManager::ModManager;
use webInterface::WebInterface;

fn main(){
    //===================Log===========================
    let log=match Log::new(){
        Ok( l ) => l,
        Err( msg )=>{
            println!( "[ERROR]Can not create log: {}", msg);
            return;
        },
    };

    //===================ServerConfig==================

    let serverConfig=match ServerConfig::read(){
        Ok( sc )=>{
            log.print(format!("[INFO]Server configurations are loaded"));
            sc
        },
        Err( msg )=>{
            log.print(format!("[ERROR]Can not read server configurations: {}", msg));
            return;
        },
    };

    //===================AppData======================
    let appData=Arc::new( AppData::new(serverConfig, log) );

    //===================ScanMods=================
    appData.log.print(format!("[INFO]Scanning existing mods"));

    match ModManager::initialize( appData.clone() ){
        Ok( _ ) => appData.log.print(format!("[INFO]Existing mods are scanned")),
        Err( msg ) => {
            appData.log.print(format!("[ERROR]Can not scan mods : {}", msg ));
            return;
        },
    }

    //===================checkAndActivateMods=================
    appData.log.print(format!("[INFO]Activating mods"));

    //appData.getModManager().checkAndActivate();

    match *appData.modManager.read().unwrap(){
        Some( ref modManager) => {
            match modManager.checkAndActivate( ){
                Ok( _ ) => appData.log.print(format!("[INFO]Mods have been checked and activated")),
                Err( msg ) => {
                    appData.log.print(format!("[ERROR]Can not check mods : {}", msg ));
                    return;
                },
            }
        },
        None=>{},
    }

    //===================WebInterface=================
    appData.log.print(format!("[INFO]Starting web interface"));

    match WebInterface::run( appData.clone() ) {
        Ok( _ ) => appData.log.print(format!("[INFO]Web interface is ready, connect to localhost:{} by your browser",appData.serverConfig.server_adminPort)),
        Err( e ) => {
            appData.log.print(format!("[ERROR]Can not run web server on port {} : {}", appData.serverConfig.server_adminPort, e.description()));
            return;
        },
    }

    //===================ConsoleInterface============
    consoleInterface::readInput( appData.clone() );

    //===================Exit========================

    appData.exit();
}

/*
fn main(){
    let appData=Arc::new( AppData::new() );
    let barrier = Arc::new(Barrier::new(2));

    let consoleInterfaceThread=consoleInterface::run( appData.clone(), barrier.clone() );
    let webInterfaceThread=consoleInterface::run( appData.clone(), barrier.clone() );

    consoleInterfaceThread.join();
    webInterfaceThread.join();





    let mut threads=Vec::new();
    let barrier = Arc::new(Barrier::new(3));



    for i in 0..4{
        let rootModule=rootModule.clone();
        let compilerData=compilerData.clone();
        let b=barrier.clone();

        threads.push(thread::spawn(move || {
            loop {
                match compilerData.modulesToCompile.lock().unwrap().pop_back() {
                    Some( (path,name) ) => {
                        let mut modulesToCompile=Vec::new();
                        let module=Module::readAndParse(&path, &name, &compilerData, &mut modulesToCompile);

                        match module {
                            Some(module)=>{
                                let mut iter=path.path.iter();

                                match iter.next(){
                                    Some( ref m )=>
                                        if *m!="" {
                                            compilerData.errorsLog.lock().unwrap().print(&format!("Path {} must begin with \"\"", path.printAsString()) );
                                        },
                                    None=>
                                        compilerData.errorsLog.lock().unwrap().print(&format!("Path for module {} must begin with \"\"", &name) ),
                                }

                                rootModule.addModule( &mut iter, &name, module, &compilerData);

                                for &(ref path, ref name) in modulesToCompile.iter(){
                                    compilerData.modulesToCompile.lock().unwrap().push_front( (path.clone(), name.clone()) );
                                }
                            },
                            None=>{},
                        }
                    },
                    None=>break,
                }
            }

            b.wait();
        }));
    }

    for t in threads{
        t.join().unwrap();
    }
}
*/



/*
extern crate nanomsg;
use std::io::Write;

use nanomsg::{Socket, Protocol, Error};

fn pusher() -> Result<(), Error> {
    let mut socket = try!(Socket::new(Protocol::Push));
    let mut endpoint = try!(socket.connect("ipc:///tmp/pipeline.ipc"));

    socket.write(b"message in a bottle");

    endpoint.shutdown();
    Ok(())
}

fn main() {
    println!("Hello, world!");

    match pusher() {
        Ok(_)=>println!("Ok"),
        Err(_)=>println!("Err"),
    }
}
*/

/*
use std::thread;

extern crate iron;
extern crate router;
extern crate rustc_serialize;

use std::io::Read;

use iron::prelude::*;
use iron::status;
use router::Router;
use rustc_serialize::json;

#[derive(RustcEncodable, RustcDecodable)]
struct Greeting {
    msg: String
}

fn main() {
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

    let mut guard=Iron::new(router).http("localhost:3000").unwrap();

    println!("aaaa");

    //guard.close().unwrap();

    //Iron::new(router).http("localhost:3000").unwrap();

    println!("aaaa");
}
*/
