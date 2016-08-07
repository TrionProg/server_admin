
use rustc_serialize::json;

use std::error::Error;

use std::io;
use std::io::prelude::*;
use std::fs::File;

#[derive(RustcDecodable, RustcEncodable)]
pub struct ServerConfig{
    pub server_adminPort:u16,
    pub server_gamePort:u16,
}

impl ServerConfig{
    pub fn read() -> Result<ServerConfig, String> {
        //let a= ServerConfig{server_adminPort:1945, server_gamePort:1941};
        //println!("{}",json::encode(&a).unwrap());

        let mut configFile=match File::open("serverConfig.cfg") {
            Ok( cf ) => cf,
            Err( e ) => return Err(format!("Can not read file \"serverConfig.cfg\" : {}", e.description())),
        };

        let mut content = String::new();
        match configFile.read_to_string(&mut content){
            Ok( c )  => {},
            Err( e ) => return Err(format!("Can not read file \"serverConfig.cfg\" : {}", e.description())),
        }

        let serverConfig: ServerConfig = match json::decode(&content){
            Ok( sc ) => sc,
            Err( e ) => return Err(format!("Can not parse file \"serverConfig.cfg\" : {}", e.description())),
        };

        Ok(serverConfig)
    }
}
