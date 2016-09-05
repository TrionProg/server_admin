
use rustc_serialize::json;

use std::error::Error;

use std::io;
use std::io::prelude::*;
use std::fs::File;

use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use config;

pub struct ServerConfig{
    pub server_adminPort:u16,
    pub server_gamePort:u16,
    pub server_address:String,
    pub repositories:RwLock<Vec<String>>,
}

impl ServerConfig{
    pub fn read() -> Result<ServerConfig, String> {

        let mut configFile=match File::open("serverConfig.cfg") {
            Ok( cf ) => cf,
            Err( e ) => return Err(format!("Can not read file \"serverConfig.cfg\" : {}", e.description())),
        };

        let mut content = String::new();
        match configFile.read_to_string(&mut content){
            Ok( c )  => {},
            Err( e ) => return Err(format!("Can not read file \"serverConfig.cfg\" : {}", e.description())),
        }

        let serverConfig: ServerConfig = match config::parse( &content, |root| {

            Ok(
                ServerConfig{
                    server_adminPort:try!(root.getStringAs::<u16>("server.adminPort")),
                    server_gamePort:try!(root.getStringAs::<u16>("server.gamePort")),
                    server_address:try!(root.getString("server.address")).clone(),
                    repositories:{
                        let repositoriesList=try!(root.getList("repositories"));

                        let mut repositories=Vec::new();

                        for repURL in repositoriesList.iter() {
                            repositories.push(try!(repURL.getString()).clone());
                        }

                        RwLock::new(repositories)
                    },
                }
            )
        }){
            Ok( sc ) => sc,
            Err( e ) => return Err(format!("Can not parse file \"serverConfig.cfg\" : {}", e)),
        };

        Ok(serverConfig)
    }
}
