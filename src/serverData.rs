use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier};

struct MainData{
    threads:Mutex< thread::Thread >,
    barrier:Barrier,
}
