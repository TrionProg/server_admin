use std::fs;
use std::fs::File;
use std::error::Error;

use std::io::{stdout, Read, Write};

use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use curl::easy::Easy as CurlDownloader;
use std::collections::VecDeque;

use appData::AppData;

fn downloadFile( URL:&String, fileName:&String ) -> Result< String, String > {
    let mut responseBytes=Vec::new();

    let mut easy = CurlDownloader::new();

    {
        try!(easy.url(&URL).or( Err(String::from("Can not assign url")) ));

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

    let fileName=format!(".tmp/{}",fileName);

    let mut file=match File::create(fileName.clone()){
        Ok ( f ) => f,
        Err( e ) => return Err(format!("Can not write file {} : {}", fileName, e.description())),
    };

    match file.write_all(&responseBytes[..]){
        Ok ( f ) => f,
        Err( e ) => return Err(format!("Can not write file {} : {}", fileName, e.description())),
    };

    Ok(fileName)
}

pub fn download( appData:&Arc<AppData>, repositories:Vec<String>, path:&str, files:VecDeque<(String, String)> ) -> Result< Vec<String>, ()> {
    let queue=Arc::new(Mutex::new(files));
    let repositories=Arc::new(repositories);
    let path=Arc::new(String::from(path));

    let result=Arc::new(Mutex::new(true));
    let downloadedFiles=Arc::new(Mutex::new(Vec::new()));

    let mut threads=Vec::new();
    let barrier = Arc::new(Barrier::new(4));

    for i in 0..4{
        let queue=queue.clone();
        let repositories=repositories.clone();
        let path=path.clone();
        let appData=appData.clone();

        let result=result.clone();
        let downloadedFiles=downloadedFiles.clone();

        let b=barrier.clone();

        threads.push(thread::spawn(move || {
            loop {
                let (firstRepURL, fileName) = match queue.lock().unwrap().pop_back() {
                    Some( (firstRepURL, fileName) ) => (firstRepURL, fileName),
                    None => break,
                };

                let mut downloaded=None;

                let URL=format!("{}{}{}",&firstRepURL,&path,&fileName);
                match downloadFile( &URL, &fileName ){
                    Ok ( f ) => downloaded=Some(f),
                    Err( e ) => {
                        appData.log.print( format!("[INFO]Can not download file from \"{}\" : {}, trying to find it at other repositories", URL, e) );

                        for repUrl in repositories.iter() {
                            let URL=format!("{}{}{}",&repUrl,&path,&fileName);

                            match downloadFile( &URL, &fileName ){
                                Ok ( f ) => {downloaded=Some(f); break;},
                                Err( e ) =>
                                    appData.log.print( format!("[INFO]{}:Cann ot download the file : {}", URL, e) ),
                            }
                        }
                    },
                }

                match downloaded{
                    Some( n ) => downloadedFiles.lock().unwrap().push(n),
                    None=>{
                        appData.log.print( format!("[ERROR]Can not download file \"{}\"", &fileName) );
                        queue.lock().unwrap().clear();
                        *result.lock().unwrap()=false;
                    }
                }
            }

            b.wait();
        }));
    }

    for t in threads{
        t.join().unwrap();
    }

    let result=result.lock().unwrap();
    if *result==false {
        let downloadedFiles=downloadedFiles.lock().unwrap();
        for fileName in downloadedFiles.iter(){
            fs::remove_file(fileName);
        }

        Err(())
    }else{
        let downloadedFiles=downloadedFiles.lock().unwrap();
        Ok(downloadedFiles.clone())
    }
}
