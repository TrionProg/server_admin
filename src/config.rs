use std::str::Chars;

use std::collections::BTreeMap;
use std::collections::btree_map::Entry::{Occupied, Vacant};

use std::str::FromStr;

pub enum ParameterValue {
    String(String),
    List(Vec<ParameterValue>),
    Map(BTreeMap<String, ParameterValue>),
}

enum ParameterClass{
    String,
    List,
    Map,
}

struct TextCursor<'a>{
    text:&'a String,
    it:Chars<'a>,
    pos:usize,
    lineBegin:usize,
    ch:char,
}

impl<'a> TextCursor<'a>{
    fn new(text:& String) -> TextCursor{
        TextCursor{
            text:text,
            it:text.chars(),
            pos:0,
            lineBegin:0,
            ch:'\0',
        }
    }

    fn next(&mut self) -> char{
        self.ch=match self.it.next(){
            None=>'\0',
            Some(ch)=>{
                if ch=='\n' {
                    self.lineBegin=self.pos;
                }

                self.pos+=1;

                ch
            }
        };

        self.ch
    }

    fn getLine( &self ) -> String{
        let mut line=String::with_capacity(80);
        for ch in self.text.chars().skip(self.lineBegin).take_while(|c| *c!='\n' && *c!='\0') {
            line.push( ch );
        }

        line
    }
}

#[derive(PartialEq, PartialOrd)]
enum Lexeme{
    EOF,
    String(String),
    Set,
    Comma,
    NewLine,
    Bracket(char),
}

impl Lexeme {
    fn next( cur:&mut TextCursor ) -> Result< Lexeme, String >{
        cur.next();

        loop {
            if cur.ch==' ' || cur.ch=='\t' {
                cur.next();
            }else if cur.ch=='/' {
                if cur.next()!='/' {
                    return Err(format!("Comment must begin with \"//\"\n{}",cur.getLine()));
                }

                while cur.ch!='\n' {
                    cur.next();
                }
            }else{
                break;
            }
        }

        match cur.ch {
            '\n'=>Ok( Lexeme::NewLine ),
            '\0'=>Ok( Lexeme::EOF ),
            ':' | '=' =>Ok( Lexeme::Set ),
            ','=>Ok( Lexeme::Comma ),
            '{' | '}' | '[' | ']' =>Ok( Lexeme::Bracket(cur.ch) ),
            '"' | '\'' => {
                let beginChar=cur.ch;
                let mut string=String::with_capacity(32);
                let mut isShielding=false;

                loop{
                    let ch=cur.next();

                    if ch=='\0' {
                        return Err(format!("Expected \"{}\" at the end of string \"{}\"\n{}",beginChar,string,cur.getLine()));
                    }else if isShielding {
                        match ch {
                            '"'=>string.push('"'),
                            '\''=>string.push('\''),
                            '\\'=>string.push('\\'),
                            'n'=>string.push('\n'),
                            _=>string.push(ch),
                        }

                        isShielding=false;
                    }else{
                        match ch {
                            '"' | '\''=>{
                                if beginChar==ch {
                                    break;
                                }else{
                                    string.push(ch);
                                }
                                string.push(ch);
                            },
                            '\\'=>isShielding=true,
                            '\n'=>return Err(format!("Expected \"{}\" at the end of \"{}\"\n{}",beginChar,string,cur.getLine())),
                            _=>string.push(ch),
                        }
                    }
                }

                Ok( Lexeme::String(string) )
            },
            _=>return Err(format!("You must write all strings in \"\"\n{}",cur.getLine())),
        }
    }
}

impl ParameterValue{
    fn parse( cur:&mut TextCursor, paramClass:ParameterClass, endsWith:char ) -> Result<ParameterValue, String>{
        match paramClass {
            ParameterClass::Map => {
                let mut params=BTreeMap::new();

                loop {
                    match try!(Lexeme::next( cur )) {
                        Lexeme::String( paramName ) => {
                            if try!(Lexeme::next( cur )) != Lexeme::Set {
                                return Err( format!("Expected : or =\n{}",cur.getLine()));
                            }

                            let paramValue=match try!(Lexeme::next( cur )) {
                                Lexeme::Bracket( '{') =>
                                    try!(ParameterValue::parse( cur, ParameterClass::Map, '}' )),
                                Lexeme::Bracket( '[') =>
                                    try!(ParameterValue::parse( cur, ParameterClass::List,']' )),
                                Lexeme::String( s ) =>
                                    ParameterValue::String( s ),
                                Lexeme::NewLine | Lexeme::Comma | Lexeme::EOF | Lexeme::Bracket('}') =>
                                    return Err(format!("Value of parameter has been missed\n{}",cur.getLine())),
                                _=>
                                    return Err( format!("Expected string(\"..\"), list([...]), map(_.._)\n{}",cur.getLine())),
                            };

                            match params.entry( paramName ) {
                                Vacant( entry ) => {entry.insert( paramValue );},
                                Occupied( e ) => return Err(format!("Parameter has been declarated before\"{}", cur.getLine())),
                            }

                            match try!(Lexeme::next( cur )){
                                Lexeme::EOF => {
                                    if endsWith!='\0' {
                                        return Err(format!("Expected _, but EOF was found\n{}",cur.getLine()));
                                    }

                                    break;
                                },
                                Lexeme::Bracket('}') => {
                                    if endsWith!='}' {
                                        return Err(format!("Expected EOF, but _ was found\n{}",cur.getLine()));
                                    }

                                    break;
                                },
                                Lexeme::NewLine | Lexeme::Comma => {},
                                _=>return Err(format!("Expected new line or , or {}\n{}",endsWith,cur.getLine())),
                            }
                        },
                        Lexeme::NewLine => {},
                        Lexeme::EOF => break,
                        _=>return Err(format!("Expected parameter name\n{}",cur.getLine())),
                    }
                }

                Ok(ParameterValue::Map(params))
            },
            ParameterClass::List => {
                let mut elements=Vec::new();

                loop{
                    let elem=match try!(Lexeme::next( cur )) {
                        Lexeme::Bracket( '{') =>
                            try!(ParameterValue::parse( cur, ParameterClass::Map, '}' )),
                        Lexeme::Bracket( '[') =>
                            try!(ParameterValue::parse( cur, ParameterClass::List,']' )),
                        Lexeme::String( s ) =>
                            ParameterValue::String( s ),
                        Lexeme::NewLine | Lexeme::Comma | Lexeme::Bracket(']') =>
                            return Err(format!("Parameter has been missed\n{}",cur.getLine())),
                        _=>
                            return Err( format!("Expected string(\"..\"), list([...]), map(_.._)\n{}",cur.getLine())),
                    };

                    match try!(Lexeme::next( cur )){
                        Lexeme::Bracket(']') =>
                            break,
                        Lexeme::NewLine | Lexeme::Comma => {},
                        _=>return Err(format!("Expected new line or , or {}\n{}",endsWith,cur.getLine())),
                    }
                }

                Ok( ParameterValue::List(elements))
            },
            ParameterClass::String => panic!("Can not parse string separatelly"),
        }
    }

    /*

    pub fn findByName<'a>(&'a self, name:String) -> Option<&'a ParameterValue> {
        match *self{
            ParameterValue::Map( ref map ) =>
                map.get(name),
            _=>None,
        }
    }

    pub fn getParameter<'a>(&'a self, name:&'a str) -> Result<Parameter<'a>,String>{
        match *self{
            ParameterValue::Map( ref map ) =>{
                match map.get(name) {
                    Some( ref pv ) => return Parameter{ name:name, value:pv },
                    None => return Err(format!("Parameter {} does not exists"))
                }
            },
            _=>return Err(format!("Not a map")),
        }
    }

    */

    /*
    fn getMap(& self) -> Result<Map,String>{
        match *self{
            ParameterValue::Map( ref params )=>
                Ok(Map{name:"root", params:params} ),
            _=>Err(format!("Not a map")),
        }
    }
    */

    fn getMap( &self) -> Map{
        match *self{
            ParameterValue::Map( ref params )=>
                Map{name:"root", params:params},
            _=>panic!("Not a map"),
        }
    }
}

pub struct Map<'a>{
    name:&'a str,
    params:&'a BTreeMap<String, ParameterValue>,
}

impl <'a>Map<'a> {
    pub fn getMap( &'a self, name:&'a str ) -> Result<Map<'a>,String>{
        match self.params.get( name ){
            Some( pv ) => {
                match *pv {
                    ParameterValue::Map( ref params ) =>
                        Ok(Map{name:name, params:params} ),
                    _=>Err(format!("Parameter \"{}\" is not map", name)),
                }
            },
            None=>Err(format!("Map \"{}\" has no parameter with name \"{}\"",self.name,name)),
        }
    }

    pub fn getString( &'a self, name:&'a str ) -> Result<&'a String,String>{
        match self.params.get( name ){
            Some( pv ) => {
                match *pv {
                    ParameterValue::String( ref string ) =>
                        Ok(string),
                    _=>Err(format!("Parameter \"{}\" is not string", name)),
                }
            },
            None=>Err(format!("Map \"{}\" has no parameter with name \"{}\"",self.name,name)),
        }
    }

    pub fn getAs<T>( &'a self, name:&'a str ) -> Result<T,String> where T: FromStr{
        let value=try!(self.getString(name));
        match value.parse::<T>() {
            Ok( p ) => Ok( p ),
            Err( e )=>Err( format!("Can not parse parameter\"{}\"", name)),
        }
    }
}

pub fn parse<T,F>( text:&String, process:F) -> Result<T, String> where F:FnOnce(Map) -> Result<T, String>{
    let mut cur=TextCursor::new(text);

    let pv=try!(ParameterValue::parse( &mut cur, ParameterClass::Map, '\0'));
    let map=pv.getMap();

    process(map)
}
