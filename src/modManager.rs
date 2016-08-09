use std::fs;
use std::error::Error;

use std::io::{stdout, Write};
use curl::easy::Easy;

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use appData::AppData;


pub struct Mod{
    name:String,
    version:Vec<u32>,
    dependencies:Vec<String>,
}

pub struct ModManager{
    pub appData:Weak<AppData>,
    pub installedMods:RwLock< HashMap<String,Mod> >,
}

impl ModManager{
    pub fn initialize( appData:Arc<AppData> ) -> Result<(),String>{
/*
        let mut easy = Easy::new();
        easy.url("https://www.rust-lang.org/").unwrap();
        easy.write_function(|data| {
            Ok(stdout().write(data).unwrap())
        }).unwrap();
        easy.perform().unwrap();

        println!("RC:{}", easy.response_code().unwrap());

        use std::process::{Command, Stdio};

Command::new("yes")
        //.arg("-l")
        //.arg("-a")
        .stdout(Stdio::null())
        .spawn()
        .expect("yes command failed to start");

        use iron::crypto::digest::Digest;
use iron::crypto::sha2::Sha256;

// create a Sha256 object
let mut hasher = Sha256::new();

// write input message
hasher.input_str("hello world");

// read hash digest
let hex = hasher.result_str();

assert_eq!(hex,
           concat!("b94d27b9934d3e08a52e52d7da7dabfa",
                   "c484efe37a5380ee9088f7ace2efcde9"));
*/

        let installedModsList=match fs::read_dir("./Mods/"){
            Ok( list ) => list,
            Err( e ) => return Err(format!("Can not read existing mods from directory Mods : {}", e.description() )),
        };

        for m in installedModsList {
            println!("Name: {}", m.unwrap().path().display())
        }

        let mut installedMods=HashMap::new();

        let modManager=Arc::new(
            ModManager{
                appData:Arc::downgrade(&appData),
                installedMods:RwLock::new(installedMods),
            }
        );

        *appData.modManager.write().unwrap()=Some(modManager.clone());

        Ok(())
    }
}
