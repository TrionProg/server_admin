
use std::io::prelude::*;
use std::fs::File;

use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use std::error::Error;

use webInterface::WebInterface;

pub struct Log{
    pub logFile:Mutex<File>,
    pub webInterface:RwLock<Option< Arc<WebInterface> >>,
}

impl Log{
    pub fn new() -> Result<Log, String> {
        let fileName=format!("Logs/log.txt");

        let mut file=match File::create(fileName.clone()){
            Ok( cf ) => cf,
            Err( e ) => return Err(format!("Can not write file {} : {}", fileName, e.description())),
        };

        match file.write_all("[LOG]\n".as_bytes()){
            Ok( cf ) => cf,
            Err( e ) => return Err(format!("Can not write file {} : {}", fileName, e.description())),
        };

        Ok(Log{
            logFile:Mutex::new(file),
            webInterface:RwLock::new( None ),
        })
    }

    pub fn print(&self, text:String){
        {
            let mut logFile=self.logFile.lock().unwrap();
            logFile.write_all(text.as_bytes());
            logFile.write("\n".as_bytes());
        }
        //self.logFile.lock().unwrap().write_all(text.as_bytes());


        match *self.webInterface.read().unwrap(){
            Some( ref wi ) => {
                if *wi.consoleIsActive.read().unwrap() {
                    let mut consoleText=wi.consoleText.write().unwrap();
                    consoleText.push_str(text.as_str());
                    consoleText.push('\n');
                }
            },
            None=>{},
        }

        println!("{}",text);
    }
}
