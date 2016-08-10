use std::fs;
use std::fs::File;
use std::error::Error;

use rustc_serialize::json;

use std::io::{stdout, Read};
use curl::easy::Easy;

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use std::path::{Path,PathBuf};
use zip;

use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use appData::AppData;
use version::Version;

#[derive(RustcDecodable, RustcEncodable)]
pub struct ModDescription{
    name:String,
    version:String,
    gameVersion:String,
    description:String,
    dependencies:Vec<String>,
}

pub struct Mod{
    pub name:String,
    pub version:Version,
    pub gameVersion:Version,
    pub dependencies:Vec< (String,Version) >,
    pub isArchive:bool,
    pub isActive:bool,
}

impl Mod{
    fn readDescription( appData: &Arc<AppData>, modPath: PathBuf ) -> Result<Mod,String> {
        //=====================Mod Name========================

        let modName=match modPath.file_name(){
            Some( n )=>{
                match n.to_str() {
                    Some( name ) => {
                        if name.ends_with(".zip") {
                            let mut n=String::from(name);
                            n.truncate(name.len()-4);
                            n
                        }else{
                            String::from(name)
                        }
                    }
                    None => return Err((format!("Bad name of mod file"))),
                }
            },
            None => return Err((format!("Mod without name"))),
        };

        //=====================Is Archive?=====================
        //change
        let isModArchive={
            match modPath.extension(){
                Some( e )=>{
                    match e.to_str() {
                        Some( extension ) => extension=="zip",
                        None => false,
                    }
                },
                None => false,
            }
        };

        //=====================Read description================

        let descriptionFileName=format!("{}/description.json",modPath.display());

        let modDescription=if isModArchive {
            let zipFile = match File::open(&modPath) {
                Ok( f ) => f,
                Err( e ) => return Err(format!("Can not read mod \"{}\" : {}", modPath.display(), e.description())),
            };

            let mut archive = match zip::ZipArchive::new(zipFile){
                Ok( a ) => a,
                Err( e ) =>return Err(format!("Can not read archive \"{}\" : {}", modPath.display(), e.description())),
            };

            let mut descriptionFile = match archive.by_name("test/description.json"){
                Ok( f ) => f,
                Err( _ ) => return Err(format!("Archive \"{}\" has no file description.json", modPath.display())),
            };

            let mut content = String::new();
            match descriptionFile.read_to_string(&mut content){
                Ok( c )  => {},
                Err( e ) => return Err(format!("Can not read file \"{}\" : {}", descriptionFileName, e.description())),
            }

            let modDescription: ModDescription = match json::decode(&content){
                Ok( d ) => d,
                Err( e ) => return Err(format!("Can not decode file \"{}\" : {}", descriptionFileName, e.description())),
            };

            modDescription
        }else{
            let mut descriptionFile=match File::open(descriptionFileName.as_str()) {
                Ok( f ) => f,
                Err( e ) => return Err(format!("Can not read file \"{}\" : {}", descriptionFileName, e.description())),
            };

            let mut content = String::new();
            match descriptionFile.read_to_string(&mut content){
                Ok( c )  => {},
                Err( e ) => return Err(format!("Can not read file \"{}\" : {}", descriptionFileName, e.description())),
            }

            let modDescription: ModDescription = match json::decode(&content){
                Ok( d ) => d,
                Err( e ) => return Err(format!("Can not decode file \"{}\" : {}", descriptionFileName, e.description())),
            };

            modDescription
        };

        //====================Name==============================

        if modDescription.name!=modName {
            return Err( format!("Mod \"{}\" has different names of its file and name in description.json",modPath.display()));
        }

        //====================Version===========================

        let version=match Version::parse( &modDescription.version ){
            Ok( v ) => v,
            Err( msg ) => return Err( format!("Can not parse version of mod \"{}\" : {}", modName, msg)),
        };

        let gameVersion=match Version::parse( &modDescription.gameVersion ){
            Ok( v ) => v,
            Err( msg ) => return Err( format!("Can not parse version of game for mod \"{}\" : {}", modName, msg)),
        };

        //====================Dependencies======================

        let mut dependencies=Vec::new();

        for dependence in modDescription.dependencies{
            let mut it=dependence.split('-');
            let nameAndVersion:Vec<&str>=dependence.split('-').collect();
            if nameAndVersion.len()!=2 {
                return Err( format!("Name of dependence mod \"{}\" for mod \"{}\" is invalid - expected format <name of mod>-<version>", dependence, modName));
            }

            let depModVersion=match Version::parse( &nameAndVersion[1].to_string() ){
                Ok( v ) => v,
                Err( msg ) => return Err( format!("Can not parse version of dependence mod \"{}\" for mod \"{}\" : {}", dependence, modName, msg)),
            };

            dependencies.push( (nameAndVersion[0].to_string(), depModVersion));
        }

        Ok(
            Mod{
                name:modName,
                version:version,
                gameVersion:gameVersion,
                dependencies:dependencies,
                isArchive:isModArchive,
                isActive:false,
            }
        )
    }
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

        /*
        for m in installedModsList {
            println!("Name: {}", m.unwrap().path().display())
        }
        */

        let mut installedMods=HashMap::new();

        for m in installedModsList {
            let modPath=m.unwrap().path();//.display().to_string();
            //let modName=modPath.file_name();
            //return modPath.file_name().unwrap().to_str().unwrap();

            /*
            match Mod::readDescription( &appData, modPath ) {
                Ok( _ ) => //insert,
                Err( msg ) => appData.log.print( msg ),
            }
            */

            //use std::fmt::Display;
            //use std::ffi::OsStr;
            //println!("{}",modName);

            //let isModZip=modPath.extension().unwrap().to_string() = "zip";
            //let extension=modPath.extension();

            /*
            let modName=modPath.split('/').last().unwrap();
            let isModZip=modName.ends_with(".zip");

            if isModZip{

            }
            */

            //println!("Name: {}",modPath.to_string());
            //return modPath.path.iter().last();
        }





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
