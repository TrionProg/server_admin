use std::fs;
use std::fs::File;
use std::error::Error;

use rustc_serialize::json;

use std::io::{stdout, Read};
use curl::easy::Easy;

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashSet;

use std::path::{Path,PathBuf};
use zip;
use std::collections::VecDeque;

use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use appData::AppData;
use version::Version;
use config;

pub struct ModDescription{
    name:String,
    version:Version,
    gameVersion:Version,
    description:String,
    dependencies:Vec< (String,Version) >,
}

impl ModDescription {
    fn read( text:&String ) -> Result<ModDescription, String> {
        let modDescription: ModDescription = try!(config::parse( text, |root| {
            Ok(
                ModDescription{
                    name:try!(root.getString("name")).clone(),
                    version:match Version::parse( try!(root.getString("version")) ) {
                        Ok( v ) => v,
                        Err( msg ) => return Err( format!("Can not parse version of mod : {}", msg)),
                    },
                    gameVersion:match Version::parse( try!(root.getString("game version")) ) {
                        Ok( v ) => v,
                        Err( msg ) => return Err( format!("Can not parse version of game for mod : {}", msg)),
                    },
                    description:try!(root.getString("description")).clone(),
                    dependencies:{
                        let depList=try!( root.getList("dependencies") );
                        let mut dependencies=Vec::new();

                        for dep in depList.iter() {
                            let dependence=try!( dep.getString() );

                            let mut it=dependence.split('-');
                            let nameAndVersion:Vec<&str>=dependence.split('-').collect();
                            if nameAndVersion.len()!=2 {
                                return Err( format!("Name of dependence mod \"{}\" is invalid - expected format <name of mod>-<version>", dependence));
                            }

                            let depModVersion=match Version::parse( &nameAndVersion[1].to_string() ){
                                Ok( v ) => v,
                                Err( msg ) => return Err( format!("Can not parse version of dependence mod \"{}\": {}", dependence, msg)),
                            };

                            dependencies.push( (nameAndVersion[0].to_string(), depModVersion));
                        }

                        dependencies
                    },
                }
            )
        }));

        Ok(modDescription)
    }
}

pub struct Mod{
    description:ModDescription,
    pub isInstalled:bool,
    pub isActive:bool,
}

impl Mod{
    fn readDescriptionFile( appData: &Arc<AppData>, descriptionFileName:&String ) -> Result<Mod, String>{
        let mut descriptionFile=match File::open(descriptionFileName.as_str()) {
            Ok( f ) => f,
            Err( e ) => return Err(format!("Can not read mod description file \"{}\" : {}", descriptionFileName, e.description())),
        };

        let mut content = String::new();
        match descriptionFile.read_to_string(&mut content){
            Ok( c )  => {},
            Err( e ) => return Err(format!("Can not read mod description file \"{}\" : {}", descriptionFileName, e.description())),
        }

        let modDescription = match ModDescription::read( &content ){
            Ok( d ) => d,
            Err( msg ) => return Err(format!("Can not decode mod description file \"{}\" : {}", descriptionFileName, msg)),
        };

        Ok(Mod{
            description:modDescription,
            isInstalled:false,
            isActive:false,
        })
    }

    fn readInstalledModDescription( appData: &Arc<AppData>, modPath: PathBuf ) -> Result<Mod,String> {
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

        let descriptionFileName=format!("{}/mod.description",modPath.display());

        let modDescription=if isModArchive {
            let zipFile = match File::open(&modPath) {
                Ok( f ) => f,
                Err( e ) => return Err(format!("Can not read mod \"{}\" : {}", modPath.display(), e.description())),
            };

            let mut archive = match zip::ZipArchive::new(zipFile){
                Ok( a ) => a,
                Err( e ) =>return Err(format!("Can not read archive \"{}\" : {}", modPath.display(), e.description())),
            };

            let mut descriptionFile = match archive.by_name("test/mod.description"){
                Ok( f ) => f,
                Err( _ ) => return Err(format!("Archive \"{}\" has no file mod.description", modPath.display())),
            };

            let mut content = String::new();
            match descriptionFile.read_to_string(&mut content){
                Ok( c )  => {},
                Err( e ) => return Err(format!("Can not read file \"{}\" : {}", descriptionFileName, e.description())),
            }

            let modDescription = match ModDescription::read( &content ){
                Ok( d ) => d,
                Err( msg ) => return Err(format!("Can not decode mod description file \"{}\" : {}", descriptionFileName, msg)),
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

            let modDescription = match ModDescription::read( &content ){
                Ok( d ) => d,
                Err( msg ) => return Err(format!("Can not decode mod description file \"{}\" : {}", descriptionFileName, msg)),
            };

            modDescription
        };

        //====================Check==============================

        if modDescription.name!=modName {
            return Err( format!("Mod \"{}\" has different names of its file and name in mod.description",modPath.display()));
        }

        //game version

        Ok(
            Mod{
                description:modDescription,
                isInstalled:true,
                isActive:false,
            }
        )
    }
}

pub struct ModManager{
    pub appData:Weak<AppData>,
    pub installedMods:RwLock< HashMap<String,Mod> >,
    pub activeMods:RwLock< Vec<String> >,
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

        //========================Installed mods========================

        let mut installedMods=HashMap::new();
        let mut modErrors=String::with_capacity(256);

        let installedModsList=match fs::read_dir("./Mods/"){
            Ok( list ) => list,
            Err( e ) => return Err(format!("Can not read existing mods from directory Mods : {}", e.description() )),
        };

        for m in installedModsList {
            let modPath=m.unwrap().path();

            match Mod::readInstalledModDescription( &appData, modPath ) {
                Ok( m ) => {
                    match installedMods.entry( m.description.name.clone() ){
                        Vacant( e ) => {e.insert( m );},
                        Occupied(_) => modErrors.push_str(format!("Mod {} have more than one packages",m.description.name).as_str()),
                    }
                },
                Err( msg ) => {
                    modErrors.push_str( msg.as_str() );
                    modErrors.push('\n');
                }
            }
        }

        if modErrors.len()>0 {
            modErrors.insert(0,'\n');
            return Err(modErrors);
        }

        //========================Active mods===========================

        let activeModsFileName="activeMods.list";

        let mut file=match File::open(activeModsFileName) {
            Ok( f ) => f,
            Err( e ) => return Err(format!("Can not read file \"{}\" : {}", activeModsFileName, e.description())),
        };

        let mut content = String::new();
        match file.read_to_string(&mut content){
            Ok( c )  => {},
            Err( e ) => return Err(format!("Can not read file \"{}\" : {}", activeModsFileName, e.description())),
        }

        let activeMods:Vec<String>=match config::parse( &content, |root| {
            let activeModsList=try!( root.getList("active mods") );
            let mut activeMods:Vec<String>=Vec::new();

            for mname in activeModsList.iter() {
                activeMods.push( try!(mname.getString()).clone() );
            }

            Ok(activeMods)
        }){
            Ok( am ) => am,
            Err( msg ) => return Err(format!("Can not decode file \"{}\" : {}", activeModsFileName, msg)),
        };

        let modManager=Arc::new(
            ModManager{
                appData:Arc::downgrade(&appData),
                installedMods:RwLock::new(installedMods),
                activeMods:RwLock::new(activeMods),
            }
        );

        *appData.modManager.write().unwrap()=Some(modManager.clone());

        Ok(())
    }

    pub fn checkAndActivate( &self ) -> Result<(), String> {
        let installedMods=&mut self.installedMods.write().unwrap();
        let activeMods=&self.activeMods.read().unwrap();

        let mut notInstalledMods=HashSet::new();
        let mut outOfDatedMods=HashSet::new();

        let mut activateMods=VecDeque::with_capacity(activeMods.len());

        for m in activeMods.iter() {
            activateMods.push_front( m.clone() );
        }

        for modName in activateMods.pop_back() {
            let addDeps=match installedMods.get_mut( &modName ){
                Some( ref mut m ) => {
                    if !m.isActive {
                        m.isActive=true;
                        true
                    }else{
                        false
                    }
                },
                None => {
                    notInstalledMods.insert( modName.clone() );
                    false
                },
            };

            if addDeps {
                match installedMods.get( &modName ){
                    Some( ref m ) => {
                        //game version
                        for &(ref depModName, ref depModVersion) in m.description.dependencies.iter() {
                            match installedMods.get( depModName ){
                                Some( ref m ) => {
                                    if m.description.version.isNewer( depModVersion ) {
                                        activateMods.push_front( depModName.clone() );
                                    }else{
                                        outOfDatedMods.insert( depModName.clone() );
                                    }
                                },
                                None => {
                                    notInstalledMods.insert( depModName.clone() );
                                },
                            }
                        }
                    },
                    None => {},
                }
            }
        }

        if notInstalledMods.len()>0 || outOfDatedMods.len() >0 {
            let mut checkErrors=String::new();
            checkErrors.push('\n');

            for nim in notInstalledMods {
                checkErrors.push_str( &format!("Mod \"{}\" is not installed yet\n",nim) );
            }

            for oodm in outOfDatedMods {
                checkErrors.push_str( &format!("Mod \"{}\" is out of date\n",oodm) );
            }

            return Err(checkErrors);
        }

        Ok(())
    }

    pub fn installMod(&self, nameOfMod:&String) -> Result<(), String> {
        Ok(())
    }
}
