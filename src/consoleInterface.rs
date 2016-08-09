
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use appData::AppData;
use std::io;

use commandProcessor;

pub fn readInput( appData:Arc<AppData> ){
    loop{
        let mut input = String::with_capacity(80);

        match io::stdin().read_line(&mut input) {
            Ok( _ ) => {
                if input.trim()=="exit" {
                    return;
                }else{
                    match commandProcessor::process( &appData, &input ){
                        Ok(_) => {},
                        Err( msg ) => appData.log.print(format!("[ERROR]{}",msg)),
                    }
                }
            },
            Err(error) => appData.log.print(format!("[ERROR]Console input error: {}",error)),
        }
    }
}
