use std::fs;
use std::fs::File;
use std::error::Error;

use rustc_serialize::json;

use std::io::{stdout, Read, Write};
//use curl::easy::Easy;

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

use curl::easy::Easy as CurlDownloader;

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
    pub fileName:String,
    pub isInstalled:bool,
    pub isActive:bool,
}

impl Mod{
    /*
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
    */

    fn readInstalledModDescription( appData: &Arc<AppData>, modPath: PathBuf ) -> Result<Mod,String> {
        //=====================Mod Name========================

        let modFileName=match modPath.file_name(){
            Some( n )=>{
                match n.to_str() {
                    Some( name ) => {
                        /*
                        if name.ends_with(".zip") {
                            let mut n=String::from(name);
                            n.truncate(name.len()-4);
                            n
                        }else{
                            String::from(name)
                        }*/
                        String::from(name)
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

        if !modFileName.starts_with(&modDescription.name) {
            return Err( format!("Mod \"{}\" has different names of its file and name in mod.description",modPath.display()));
        }

        //game version

        Ok(
            Mod{
                description:modDescription,
                fileName:modFileName.clone(),
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
    solution:RwLock<Option< Solution > >,
}

impl ModManager{
    pub fn initialize( appData:Arc<AppData> ) -> Result<(),String>{
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

        //========================Activate mods===========================

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
                solution:RwLock::new(None),
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
                                    if m.description.version>=*depModVersion {
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

    fn downloadAndReadDescription(repURL:&String, modName:&String, modVersion:&Option<Version> ) -> Result<ModDescription, String>{
        //================================Make URL===========================
        let mut requestURL=format!("{}/mods/description/{}",repURL,modName);
        //let mut requestURL=format!("{}/mods/description/{}?gameVersion={}",&repositoryURL,&modName,GAME_VERSION.print());
        match *modVersion {
            Some( ref modVersion ) => requestURL.push_str(&format!("_modVersion={}",modVersion.print())),
            None => {},
        }

        //================================Download=============================

        let mut responseBytes=Vec::new();

        let mut easy = CurlDownloader::new();

        {
            try!(easy.url(&requestURL).or( Err(String::from("Can not assign url")) ));

            let mut transfer = easy.transfer();
            transfer.write_function(|data| {
                &responseBytes.extend_from_slice(data);
                Ok(data.len())
            });

            try!(transfer.perform().or(Err(String::from("Can not perform"))));
        }

        {
            let statusCode=easy.response_code().unwrap();
            if statusCode!=200 {
                return Err(format!("Can not download. Status : {}", statusCode));
            }
        }

        //================================To UTF-8===========================

        let descriptionText=try!(String::from_utf8(responseBytes).or(Err(String::from("description is no valid UTF-8 file"))));

        let modDescription=match ModDescription::read(&descriptionText){
            Ok ( md ) => md,
            Err( e ) => return Err(format!("Can not read description : {}", e)),
        };

        //================================Check==============================

        //check game version,name

        match *modVersion {
            Some( ref modVersion ) => {
                if modDescription.version<*modVersion {
                    return Err(format!("Version is old \"{}\"", modDescription.version.print()));
                }
            },
            None => {},
        }

        Ok( modDescription )
    }

    pub fn installMods(&self, modList:Vec<(String,Option<Version>)>) -> Result<(), String> {
        let appData=self.appData.upgrade().unwrap();

        //=======================Solve dependencies=========================

        let mut installModList:VecDeque<(String, Option<Version>)> = VecDeque::new();
        for &(ref modName, ref modVersion) in modList.iter(){
            installModList.push_front( (modName.clone(), modVersion.clone() ) );
        }
        //installModList.push_front( (String::from(nameOfMod), /*None*/Some(Version::parse( &String::from("0.1.2.0")).unwrap() )) );

        let mut virtInstalledMods=HashMap::new();

        {
            let installedMods=self.installedMods.read().unwrap();

            for (modName, modData) in (*installedMods).iter(){
                virtInstalledMods.insert(modName.clone(), (String::from("localhost"), modData.description.version.clone()) );
            }
        }

        loop{
            let (modName, modVersion)=match installModList.pop_back(){
                Some( mnv ) => mnv,
                None => break,
            };

            let mut modDescriptionAndRepURL=None;

            let repositories=appData.serverConfig.repositories.read().unwrap();

            for repositoryURL in (*repositories).iter() {
                match ModManager::downloadAndReadDescription(repositoryURL, &modName, &modVersion) {
                    Ok ( d ) => {
                        modDescriptionAndRepURL=Some( (d,repositoryURL.clone()) ); break;
                    },
                    Err( e ) => {
                        let versionStr=match modVersion{
                            Some(ver) => ver.print(),
                            None => String::from("compatible with game version"),
                        };

                        appData.log.print(format!("[INFO]rep:\"{}\", mod:\"{}\", ver:\"{}\" : {}", repositoryURL, &modName, versionStr, e));
                    },
                }
            }

            match modDescriptionAndRepURL {
                Some( (modDescription, repURL) ) => {
                    virtInstalledMods.insert( modName, (repURL, modDescription.version.clone()) );

                    for &(ref depName, ref depVersion) in modDescription.dependencies.iter() {
                        match virtInstalledMods.get(depName) {
                            Some( &( _ , ref instDepVersion) ) => {
                                if *instDepVersion<*depVersion {
                                    installModList.push_front( (depName.clone(), Some( depVersion.clone()) ) );
                                }
                            },
                            None => installModList.push_front( (depName.clone(), Some( depVersion.clone()) ) ),
                        }
                    }
                },
                None => {appData.log.print(format!("[ERROR]Can not download description for mod \"{}\"",&modName)); return Ok(());},//false
            }
        }

        //=============================Ask about solution==========================

        let mut solutionAction=Vec::new();

        {
            let installedMods=self.installedMods.read().unwrap();

            for (modName, &(ref repURL, ref modVersion) ) in virtInstalledMods.iter() {
                match installedMods.get(modName) {
                    Some( modData ) => {
                        if modData.description.version<*modVersion {
                            solutionAction.push((SolutionAction::Update, repURL.clone(), modName.clone(), modVersion.clone() ));
                        }
                    },
                    None => solutionAction.push((SolutionAction::Install, repURL.clone(), modName.clone(), modVersion.clone() )),
                }
            }
        }

        *self.solution.write().unwrap()=Some(Solution::new(solutionAction));

        match self.processSolution(){
            Ok ( _ ) => {},
            Err( e ) => appData.log.print(e),
        }

        //ask
        //appData.log.print(String::from("Write Y to contunue or N to abort"));

        Ok(())
    }

    fn processSolution(&self) -> Result<(), String> {
        let appData=self.appData.upgrade().unwrap();
        let solution=self.solution.read().unwrap();

        match *solution{
            Some( ref solution ) => {
                appData.log.print( solution.print() );
                //сначала должны скачаться файлы во временный каталог

                //===============================CREATE BACKUP DIRECTORY=========================
                let backupDirectoryName=format!("backups/backup");
                {
                    let solutionText=solution.print();

                    try!( fs::create_dir(&backupDirectoryName).or( Err(String::from("Can not create backup directory")) ));

                    let mut file=try!( File::create( &format!("{}/actions.txt",backupDirectoryName) ).or( Err(String::from("Can not create actions.txt file"))) );

                    try!( file.write_all( solutionText.as_bytes() ).or( Err(String::from("Can not write to file actions.txt"))) );


                }

                //==============================MOVE FILES TO BACKUP DIRECTORY==================
                {
                    let installedMods=self.installedMods.read().unwrap();

                    for &(ref solutionAction, ref repURL, ref modName, ref modVersion) in solution.actions.iter(){
                        match *solutionAction {
                            SolutionAction::Update | SolutionAction::Remove => {
                                match installedMods.get(modName) {
                                    Some( modData ) => {
                                        try!(fs::rename(
                                            &format!("Mods/{}", modData.fileName),
                                            &format!("{}/{}", backupDirectoryName, modData.fileName))
                                        .or(Err(format!("Can not move file  \"{}\"",modData.fileName ))));
                                    },
                                    None => return Err( format!("Mod \"{}\" does not exists", modName) ),
                                }
                            },
                            _=>{},
                        }
                    }
                }
            },
            None=>{}
        }

        Ok(())
    }
}

#[derive(PartialEq, Eq)]
enum SolutionAction{
    Install,
    Update,
    Remove,
}

impl SolutionAction{
    fn print<'a>(&'a self) -> &'a str{
        match *self{
            SolutionAction::Install => "install",
            SolutionAction::Update => "update",
            SolutionAction::Remove => "remove",
        }
    }
}

struct Solution{
    actions:Vec< (SolutionAction, String, String, Version) >,
}

impl Solution{
    fn new(actions:Vec< (SolutionAction, String, String, Version) >) -> Solution {
        Solution{
            actions:actions,
        }
    }

    fn print(&self) -> String {
        let mut solutionText=String::with_capacity(1024);

        for &(ref solutionAction, ref repURL, ref modName, ref modVersion) in self.actions.iter(){
            solutionText.push_str( &format!("{} \"{}-{}\"\n", solutionAction.print(), modName, modVersion.print() ));
        }

        solutionText
    }
}
