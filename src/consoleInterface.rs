/*
use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier};

use appData::AppData;

pub fn run( appData:Arc< AppData>, barrier:Arc< Barrier> ) -> thread::Thread {
    thread::spawn(move || {
        process( appData );
        barrier.wait();
    })
}

fn process( appData:Arc< AppData> ){

}
*/
