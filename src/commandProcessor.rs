use std::str::Chars;

use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use appData::AppData;
use std::slice::Iter;

enum Lexeme{
    EOF,
    Word(String),
    String(String),
}

static EOF_LEX:Lexeme=Lexeme::EOF;

struct Command{
    lexemes:Vec<Lexeme>,
    lineBegin:usize,
    lineEnd:usize,
}

fn getLine( text:&String, lineBegin:usize, lineEnd:usize ) -> &str{
    let (before, after)=text.split_at(lineBegin);
    let (line, after)=after.split_at(lineEnd-lineBegin-1);

    line
}

fn runLexer( text:&String ) -> Result< Vec<Command>, String> {
    fn nextChar( pos:&mut usize, it:&mut Chars ) -> char {
        *pos+=1;

        match it.next(){
            Some( ch ) => ch,
            None => '\0',
        }
    }

    let mut commands=Vec::new();
    let mut lexemes=Vec::new();

    let mut lineBegin=0;
    let mut it=text.chars();
    let mut pos=0;

    let mut ch=nextChar( &mut pos, &mut it );

    loop{
        match ch{
            '\0' | '\n' => {
                if lexemes.len()>0 {
                    commands.push(
                        Command{
                            lexemes:lexemes,
                            lineBegin:lineBegin,
                            lineEnd:pos,
                        }
                    );

                    lexemes=Vec::new();
                }

                if ch=='\0' {
                    break;
                }

                lineBegin=pos;
                ch=nextChar( &mut pos, &mut it );
            },
            ' ' => ch=nextChar( &mut pos, &mut it ),
            '"' | '\'' => {
                let beginChar=ch;
                let mut string=String::with_capacity(32);
                let mut isShielding=false;

                loop{
                    let ch=nextChar( &mut pos, &mut it );

                    if ch=='\0' {
                        return Err(format!("Expected \"{}\" at the end of \"{}\"\n{}",beginChar,string,getLine(&text,lineBegin,pos)));
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
                            '\n'=>return Err(format!("Expected \"{}\" at the end of \"{}\"\n{}",beginChar,string,getLine(text,lineBegin,pos))),
                            _=>string.push(ch),
                        }
                    }
                }

                lexemes.push(Lexeme::String(string));

                ch=nextChar( &mut pos, &mut it );
            },
            _=>{
                let mut word=String::with_capacity(32);
                word.push(ch);

                loop {
                    ch=nextChar( &mut pos, &mut it );

                    match ch {
                        '\0' | '\n' | ' ' => break,
                        _=> word.push(ch),
                    }
                }

                lexemes.push(Lexeme::Word(word));
            },
        }
    }

    Ok(commands)
}

fn nextLexeme<'a>( it:&'a mut Iter<Lexeme> ) -> &'a Lexeme {
    match it.next(){
        Some( ref l ) => l,
        None=> &EOF_LEX,
    }
}

pub fn process( appData:&Arc<AppData>, text:&String ) -> Result<(), String> {
    let commands=try!( runLexer( text ) );

    for command in commands.iter(){
        match processCommand( command ){
            Ok(()) => {},
            Err( msg ) => return Err(format!("{}\n{}",msg, getLine( text, command.lineBegin, command.lineEnd ) ) ),
        }
    }

    Ok(())
}

fn processCommand( command:&Command ) -> Result< (), String > {
    let mut it=command.lexemes.iter();

    match *nextLexeme( &mut it ){
        Lexeme::EOF => {},
        Lexeme::String( ref s ) => return Err(format!("Can not use string \"{}\" as command",s)),
        Lexeme::Word( ref w ) => {
            match w.as_ref() {
                "run" => {
                    /*
                    match nextLexeme( &mut it ){
                        Lexeme::EOF => {},
                        Lexeme::Word( ref map ) | Lexeme::String( ref map ) => {},
                    }
                    */
                    println!("haha");
                },
                "stop" => {},
                _=>return Err(format!("Unknown command: \"{}\" ", w )),
            }
        },
    }

    return Ok(())
}