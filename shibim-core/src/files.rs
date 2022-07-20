use std::fs as fs;
use std::path::{Path};
use crate::error::{LoadError, LoadErrorPayload, VisitorError};
use shibim_base::*;
use shibim_parse::parse_song;
use std::collections::HashMap;

fn read_shb(file_name : &Path) -> 
    Result<Song,LoadError >{
    let u = fs::read_to_string(file_name);
    let u = u.map_err(|e|
        LoadError{
            file : file_name.to_string_lossy().to_string(),
            detail : LoadErrorPayload::IOError(e)
        }
    );
    u.and_then(|s|
        parse_song(&s)
        .map_err(|e|
            LoadError{
                file : file_name.to_string_lossy().to_string(),
                detail : LoadErrorPayload::ParseError(e)
            }
        )
    )
}   

fn read_shb_batch_seq<'a, I>(batch : I, visitors : Vec<&mut dyn SongVisitor>)
-> HashMap<String,Song>
where
    I : IntoIterator<Item = &'a Path>{
    let mut out = HashMap::new();
    for file in batch{
        match read_shb(file){
            Ok(song) =>{
                let err = out.insert(file.to_string_lossy().to_string(),song);
                if err.is_some(){
                    eprintln!("Files with same UTF-8 filename {}",file.display());
                }
            },
            Err(err) =>{
                eprintln!("{}",err);
            }
        }
    }
    out
}

trait SongVisitor{
    fn process(&mut self, e : &Song) -> Result<(),VisitorError>;
}