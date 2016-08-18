
use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use appData::AppData;

use iron::prelude::*;
use iron::status;
use router::Router;
use rustc_serialize::json;

use iron::error::HttpError;
use iron::Listening;
use iron::mime::Mime;

use std::io::Read;
use std::error::Error;

use std::fs::File;
use std::io;

pub struct WebInterface{
    pub appData:Weak<AppData>,
    files:Files,
    mimeTypes:MimeTypes,
    listener:Mutex<Option<Listening>>,
    pub consoleIsActive:RwLock<bool>,
    pub consoleText:RwLock<String>,
}

struct Files{
    index:RwLock<String>
}

impl Files{
    fn load() -> Result<Files, String> {
        Ok(
            Files{
                index:RwLock::new(String::from("Hello world")),
            }
        )
    }

    fn readFile( fileName:&str) -> Result<Vec<u8>, String> {
        let fileName=format!("Files/{}", fileName);

        let mut file=match File::open(&fileName) {
            Ok ( f ) => f,
            Err( e ) => return Err(format!("Can not read file \"{}\" : {}", fileName, e.description()) ),
        };

        let mut content = Vec::new();
        match file.read_to_end(&mut content){
            Ok ( c )  => {},
            Err( e ) => return Err(format!("Can not read file \"{}\" : {}", fileName, e.description()) ),
        }

        Ok(content)
    }

    fn readUTF8File( fileName:&str ) -> Result<String, String> {
        let text=match String::from_utf8( try!(Files::readFile( fileName )) ){
            Ok ( p ) => p,
            Err( _ ) => return Err( format!("File {} is no valid utf-8 file", fileName) ),
        };

        let mut resultString=String::with_capacity(16*1024);

        let mut prevIsSpace=false;

        for ch in text.chars(){
            if ch.is_whitespace() {
                if !prevIsSpace {
                    resultString.push(ch);
                    prevIsSpace=true;
                }
            }else{
                prevIsSpace=false;
                resultString.push(ch);
            }
        }

        Ok(resultString)
    }

    fn buildWebPage( pageName:&str ) -> Result<String, String> {
        use std::str::Chars;
        let page=try!( Files::readUTF8File(pageName) );

        let mut it=page.chars();

        let mut resultString=String::with_capacity(16*1024);

        fn nextChar( it:&mut Chars) -> char{
            match it.next() {
                Some( ch ) => ch,
                None => '\0',
            }
        }

        loop{
            let ch=nextChar( &mut it );
            match ch{
                '<' => {
                    let ch=nextChar( &mut it );
                    match ch{
                        '?'=> {
                            let mut fileName=String::new();

                            loop {
                                let ch=nextChar( &mut it );

                                if ch=='/' {
                                    break;
                                }else if ch=='\0'{
                                    return Err(String::from("Expected />"));
                                }

                                fileName.push(ch);
                            }

                            nextChar( &mut it );

                            let fileContent=try!(Files::readUTF8File( &fileName ));
                            resultString.push_str( &fileContent );
                        },
                        '\0'=>break,
                        _=>{
                            resultString.push('<');
                            resultString.push(ch);
                        }
                    }
                }
                '\0'=>break,
                _=>resultString.push(ch),
            }
        }

        Ok(resultString)
    }
}

struct MimeTypes{
    text:Mime,
    html:Mime,
    png:Mime,
}

impl WebInterface{
    pub fn run( appData:Arc<AppData> ) -> Result<(), String> {
        let files=try!(Files::load());

        let mimeTypes=MimeTypes{
            text:"text/plane".parse::<Mime>().unwrap(),
            html:"text/html".parse::<Mime>().unwrap(),
            png: "img/png".parse::<Mime>().unwrap(),
        };

        let webInterface=Arc::new(WebInterface{
            appData:Arc::downgrade(&appData),
            files:files,
            mimeTypes:mimeTypes,
            listener:Mutex::new(None),
            consoleIsActive:RwLock::new(false),
            consoleText:RwLock::new(String::with_capacity(256)),
        });

        match WebInterface::runHTTPListener( appData, webInterface){
            Ok ( _ ) => Ok(()),
            Err( e ) => Err(format!("Can not create HTTP listener : {}",e.description())),
        }
    }

    fn runHTTPListener( appData:Arc< AppData>, webInterface:Arc< WebInterface > ) -> Result<(), HttpError>  {
        let mut router = Router::new();

        /*
        let router_webInterface=webInterface.clone();
        router.get("/", move |r: &mut Request| Ok(Response::with(
            (status::Ok, (*router_webInterface.files.index.read().unwrap()).clone())
        )) );
        */

        let router_webInterface=webInterface.clone();
        router.get("/", move |r: &mut Request| Ok(Response::with((router_webInterface.mimeTypes.html.clone(),
            status::Ok, match Files::buildWebPage("index.html") { Ok(m) => m, Err(e)=>format!("err:{}",e),})
        )) );

        let files=vec!["background.png",  "icon_map.png",  "icon_mod.png",
                       "icon.png",  "icon_run.png",  "icon_settings.png",
                       "icon_stop.png",  "icon_x.png"];

        for f in files.iter() {
            let url=format!("/{}",f);
            let fileName=format!("Files/{}",f);
            let router_webInterface=webInterface.clone();
            router.get(url.as_str(), move |r: &mut Request| Ok(Response::with((router_webInterface.mimeTypes.png.clone(),
                status::Ok, WebInterface::contentFromFile(fileName.as_str()))
            )) );
        }

        let router_webInterface=webInterface.clone();
        router.get("/login", move |r: &mut Request| Ok(Response::with((router_webInterface.mimeTypes.text.clone(),
            status::Ok, WebInterface::contentFromFile("Files/loginAnswer.txt"))
        )) );

        /*

        router.get("/", move |r: &mut Request| hello_world(r, &greeting.lock().unwrap()));
        router.post("/set", move |r: &mut Request| set_greeting(r, &mut greeting_clone.lock().unwrap()));

        fn hello_world(_: &mut Request, greeting:&Greeting) -> IronResult<Response> {
            let greeting = Greeting { msg: "Hello, World".to_string() };
            let payload = json::encode(&greeting).unwrap();
            Ok(Response::with((status::Ok, payload)))
        }

        // Receive a message by POST and play it back.
        fn set_greeting(request: &mut Request) -> IronResult<Response> {
            let mut payload = String::new();
            request.body.read_to_string(&mut payload);
            let request: Greeting = json::decode(payload.as_ref()).unwrap();
            let greeting = Greeting { msg: request.msg };
            let payload = json::encode(&greeting).unwrap();
            Ok(Response::with((status::Ok, payload)))
        }
        */

        let address=format!("localhost:{}",appData.serverConfig.server_adminPort);
        let listener=try!(Iron::new(router).http(address.as_str()));

        *webInterface.listener.lock().unwrap()=Some(listener);

        *appData.webInterface.write().unwrap()=Some(webInterface.clone());
        *appData.log.webInterface.write().unwrap()=Some(webInterface);

        Ok( () )
    }

    fn contentFromFile( fileName:&str ) -> String {
        //use std::fs::File;
        //use std::io;

        let mut file=match File::open(fileName) {
            Ok ( f ) => f,
            Err( e ) => return String::from("NotFound"),
        };

        let mut content = Vec::new();
        match file.read_to_end(&mut content){
            Ok ( c )  => {},
            Err( e ) => return String::from("Error"),
        }

        unsafe{
            String::from_utf8_unchecked(content)
        }
    }

    pub fn close(&self){
        let appData=self.appData.upgrade().unwrap();

        appData.log.print(format!("[INFO]Closing web interface"));

        *self.consoleIsActive.write().unwrap()=false;
        *appData.log.webInterface.write().unwrap()=None;

        match *self.listener.lock().unwrap() {
            Some( ref mut listener ) => {listener.close();},
            None => {},
        }

        appData.log.print(format!("[INFO]Web interface has been closed"));
    }
}
