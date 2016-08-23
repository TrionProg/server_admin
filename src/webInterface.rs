
use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::btree_map::Entry::Occupied as BTreeMapOccupied;
use std::collections::btree_map::Entry::Vacant as BTreeMapVacant;
use std::collections::hash_map::Entry::Occupied as HashMapOccupied;
use std::collections::hash_map::Entry::Vacant as HashMapVacant;

use appData::AppData;

use iron::prelude::*;
use iron::status;
use router::Router;
use rustc_serialize::json;
use rustc_serialize::base64::{ToBase64, FromBase64, STANDARD};

use iron::error::HttpError;
use iron::Listening;
use iron::mime::Mime;

use std::io::Read;
use std::error::Error;

use std::fs::File;
use std::io;

use time;

use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::curve25519xsalsa20poly1305::PublicKey as Box_PublicKey;
use sodiumoxide::crypto::box_::curve25519xsalsa20poly1305::SecretKey as Box_SecretKey;
use sodiumoxide::crypto::box_::curve25519xsalsa20poly1305::Nonce as Box_Nonce;
use sodiumoxide::crypto::secretbox;
use sodiumoxide::crypto::secretbox::xsalsa20poly1305::Key as SecretBox_Key;
use sodiumoxide::crypto::secretbox::xsalsa20poly1305::Nonce as SecretBox_Nonce;
use sodiumoxide::randombytes::randombytes_into;
use sodiumoxide::crypto::pwhash;

struct LoginingClient{
    publicKeyB:     Box_PublicKey,
    secretKeyA:     Box_SecretKey,
    nonce:          Box_Nonce,
    time:           time::Timespec,
}

pub struct AdminSession{
    adminKey:            String,
    time:           time::Timespec,
    requestKey:     SecretBox_Key,
    requestNonce:   SecretBox_Nonce,
    responseKey:    SecretBox_Key,
    responseNonce:  SecretBox_Nonce,
    pub news:           String,
}

pub struct WebInterface{
    pub appData:Weak<AppData>,
    files:Files,
    mimeTypes:MimeTypes,
    listener:Mutex<Option<Listening>>,

    loginingClients:RwLock<HashMap<u64, LoginingClient>>,
    pub adminSession:RwLock<Option<AdminSession>>,
}

const loginingClientsLimit:usize=1024;

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
            loginingClients:RwLock::new(HashMap::new()),
            adminSession:RwLock::new(None),
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

        let router_webInterface=webInterface.clone();
        router.get("/crypto", move |r: &mut Request| Ok(Response::with((router_webInterface.mimeTypes.html.clone(),
            status::Ok, WebInterface::contentFromFile("Files/Crypto/index.html"))
        )) );

        let router_webInterface=webInterface.clone();
        router.get("/sodium.js", move |r: &mut Request| Ok(Response::with((router_webInterface.mimeTypes.text.clone(),
            status::Ok, WebInterface::contentFromFile("Files/Crypto/sodium.js"))
        )) );

        let router_webInterface=webInterface.clone();
        router.post("/login", move |r: &mut Request|
            match WebInterface::login(r,&router_webInterface) {
                Ok ( msg ) =>
                    Ok(Response::with( (router_webInterface.mimeTypes.text.clone(), status::Ok, msg) )),
                Err( msg ) => {
                    router_webInterface.appData.upgrade().unwrap().log.write(msg);
                    Ok(Response::with( (router_webInterface.mimeTypes.text.clone(), status::BadRequest, String::from(msg)) ))
                }
            }
        );

        let router_webInterface=webInterface.clone();
        router.post("/arenews", move |r: &mut Request|
            match WebInterface::checkNews(r,&router_webInterface) {
                Ok ( responseCipherBase64 ) =>
                    Ok(Response::with( (router_webInterface.mimeTypes.text.clone(), status::Ok, responseCipherBase64) )),
                Err( msg ) => {
                    router_webInterface.appData.upgrade().unwrap().log.write(msg);
                    Ok(Response::with( (router_webInterface.mimeTypes.text.clone(), status::BadRequest, String::from(msg)) ))
                }
            }
        );

        let address=format!("localhost:{}",appData.serverConfig.server_adminPort);
        let listener=try!(Iron::new(router).http(address.as_str()));

        *webInterface.listener.lock().unwrap()=Some(listener);

        *appData.webInterface.write().unwrap()=Some(webInterface.clone());
        *appData.log.webInterface.write().unwrap()=Some(webInterface);

        Ok( () )
    }

    fn randomU64() -> u64{
        use std::mem;
        let mut a=[0;8];
        randombytes_into(&mut a);
        unsafe{
            mem::transmute::<[u8; 8], u64>(a)
        }
    }

    fn generateLoginingClient<'a>( &self, fields:BTreeMap<String, String> ) -> Result<String, &'a str> {
        loop{
            let id=WebInterface::randomU64();

            match self.loginingClients.write().unwrap().entry(id.clone()) {
                HashMapVacant( e ) => {
                    let loginPublicKeyB={
                        let keyBase64=try!( fields.get("public key b").ok_or( "Public key b field does not exists" ) );
                        let keyBytes=try!( keyBase64.from_base64().or( Err("Can not decode public key b") ) );
                        try!( Box_PublicKey::from_slice(&keyBytes).ok_or( "Can not decode public key b" ) )
                    };

                    let loginNonce={
                        let nonceBase64=try!( fields.get("nonce").ok_or( "Nonce field does not exists" ) );
                        let nonceBytes=try!( nonceBase64.from_base64().or( Err("Can not decode nonce") ) );
                        try!( Box_Nonce::from_slice(&nonceBytes).ok_or( "Can not decode nonce" ) )
                    };

                    let (publicKeyA, secretKeyA) = box_::gen_keypair();

                    let loginingClient=LoginingClient{
                        publicKeyB:     loginPublicKeyB,
                        secretKeyA:     secretKeyA,
                        nonce:          loginNonce,
                        time:           time::get_time(),
                    };

                    e.insert(loginingClient);

                    let publicKeyABase64=publicKeyA[..].to_base64(STANDARD);

                    return Ok( format!("ok:{};{}",id,publicKeyABase64) );
                },
                HashMapOccupied( e ) => {},
            }
        }
    }

    fn login<'a>( req: &'a mut Request, webInterface:&Arc< WebInterface > ) -> Result<String, &'a str> {
        if webInterface.adminSession.read().unwrap().is_some() {
            return Ok( String::from("message:Other admin is online") );
        }

        let fields=try!(WebInterface::parseRequestBody(req));

        let id = {
            let idStr=try!( fields.get("id").ok_or( "Id field does not exists" ) );
            try!( idStr.parse::<u64>().or( Err( "Can not parse id") ) )
        };

        if id==0 {
            if webInterface.loginingClients.read().unwrap().len()>=loginingClientsLimit {
                return Ok( String::from("error:3DDOS attack! A lot of clients are trying to login") );
            }

            return webInterface.generateLoginingClient( fields );
        }else{
            let lc=try!( webInterface.loginingClients.write().unwrap().remove(&id).ok_or( "Id not found" ) );

            #[derive(RustcEncodable, RustcDecodable)]
            struct LoginData{
                password:String,
                requestKey:String,
                requestNonce:String,
                responseKey:String,
                responseNonce:String,
            }

            let data:LoginData={
                let cipherDataBase64=try!( fields.get("cipher data").ok_or( "Cipher data field does not exists" ) );
                let cipherDataBytes=try!( cipherDataBase64.from_base64().or( Err("Can not decode cipher data") ) );

                let jsonDataBytes=try!( box_::open(&cipherDataBytes, &lc.nonce, &lc.publicKeyB, &lc.secretKeyA).or( Err("Can not decode Data") ) );
                let jsonData=try!( String::from_utf8( jsonDataBytes).or( Err("Login Data is not valid UTF-8") ));

                try!( json::decode(&jsonData).or( Err("Can not decode Data")) )
            };

            let passwordHash=pwhash::pwhash(data.password.as_bytes(),
                pwhash::OPSLIMIT_INTERACTIVE,
                pwhash::MEMLIMIT_INTERACTIVE).unwrap();

            let passwordHashBase64=passwordHash[..].to_base64(STANDARD);

            if pwhash::pwhash_verify(&passwordHash, data.password.as_bytes()) {
                let requestKeyBytes=try!( data.requestKey.from_base64().or( Err("Can not decode request key") ));
                let requestNonceBytes=try!( data.requestNonce.from_base64().or( Err("Can not decode request nonce") ));
                let responseKeyBytes=try!( data.responseKey.from_base64().or( Err("Can not decode response key") ));
                let responseNonceBytes=try!( data.responseNonce.from_base64().or( Err("Can not decode response nonce") ));

                let mut adminKey=[0;32];
                randombytes_into(&mut adminKey);
                let adminKeyBase64=adminKey.to_base64(STANDARD);

                let adminSession=AdminSession{
                    adminKey:adminKeyBase64,
                    time:time::get_time(),
                    requestKey:try!( SecretBox_Key::from_slice( &requestKeyBytes ).ok_or( "Can not decode request key") ),
                    requestNonce:try!( SecretBox_Nonce::from_slice( &requestNonceBytes ).ok_or( "Can not decode request nonce") ),
                    responseKey:try!( SecretBox_Key::from_slice( &responseKeyBytes ).ok_or( "Can not decode response key") ),
                    responseNonce:try!( SecretBox_Nonce::from_slice( &responseNonceBytes ).ok_or( "Can not decode response nonce") ),
                    news:String::with_capacity(1024),
                };

                let adminKeyCipher=secretbox::seal(adminSession.adminKey.as_bytes(), &adminSession.responseNonce, &adminSession.responseKey);
                let adminKeyCipherBase64=adminKeyCipher.to_base64(STANDARD);

                if webInterface.adminSession.read().unwrap().is_some() {
                    return Ok( String::from("message:Other admin is online") );
                }

                *webInterface.adminSession.write().unwrap()=Some(adminSession);

                Ok(format!("ok:{}",adminKeyCipherBase64))
            }else{
                Ok( String::from("message:Incorrect Password") )
            }
        }
    }

    fn checkNews<'a>( req: &mut Request, webInterface:&'a Arc< WebInterface > ) -> Result<String, &'a str> {
        match *webInterface.adminSession.write().unwrap(){
            Some( ref mut adminSession )=>{
                let mut adminKeyBase64 = String::new();
                try!( req.body.read_to_string(&mut adminKeyBase64).or( Err("Can not read body") ));

                try!( webInterface.checkAdminSession( adminSession, adminKeyBase64 ));

                let response=if adminSession.news.len()>0 {
                    let r=format!("admin key:{};\n{}", &adminSession.adminKey, &adminSession.news);
                    adminSession.news.clear();
                    r
                }else{
                    format!("admin key:{};\n", &adminSession.adminKey)
                };

                let responseCipher=secretbox::seal(response.as_bytes(), &adminSession.responseNonce, &adminSession.responseKey);
                Ok(responseCipher.to_base64(STANDARD))
            },
            None =>
                Err("Not logined")
        }
    }

    fn checkAdminSession<'a>( &'a self, adminSession:&mut AdminSession, adminKeyBase64:String ) -> Result<(),&'a str> {
        if adminSession.adminKey!=adminKeyBase64 {
            return Err("Admin keys mistmatch");
        }

        let mut adminKey=[0;32];
        randombytes_into(&mut adminKey);
        let adminKeyBase64=adminKey.to_base64(STANDARD);

        adminSession.time=time::get_time();
        adminSession.adminKey=adminKeyBase64;

        Ok(())
    }

    fn parseRequestBody<'a>( req: &'a mut Request ) -> Result<BTreeMap<String, String>, &'a str> {
        let mut body = String::new();
        match req.body.read_to_string(&mut body){
            Ok ( _ ) => {},
            Err( e ) => return Err( "Can not read body"),
        }

        let mut it=body.chars();
        let mut fields=BTreeMap::new();

        loop{
            let mut fieldName=String::with_capacity(16);
            loop{
                match it.next(){
                    Some(':') => break,
                    Some( ch ) => fieldName.push(ch),
                    None => {
                        return if fieldName.len()==0 {
                            Ok(fields)
                        }else{
                            Err( "Unexpected EOF of body" )
                        }
                    },
                }
            }

            let mut fieldValue=String::with_capacity(32);

            loop{
                match it.next(){
                    Some('\n') => break,
                    Some( ch ) => fieldValue.push(ch),
                    None => return Err( "Unexpected EOF of body" ),
                }
            }

            match fields.entry(fieldName) {
                BTreeMapVacant( e ) => {e.insert( fieldValue );},
                BTreeMapOccupied( e ) => return Err( "Double field name" ),
            }
        }
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

        *appData.log.webInterface.write().unwrap()=None;

        match *self.listener.lock().unwrap() {
            Some( ref mut listener ) => {listener.close();},
            None => {},
        }

        appData.log.print(format!("[INFO]Web interface has been closed"));
    }
}
