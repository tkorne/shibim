use std::fs as fs;
use std::path::{Path};
use crate::error::LoadError;
use shibim_base::*;
use shibim_parse::parse_song;

fn read_shb(file_name : &Path) -> 
    Result<Song,LoadError >{
    let u = fs::read_to_string(file_name);
    let u = u.map_err(LoadError::IOError);
    u.and_then(|s|
        parse_song(&s)
        .map_err(|_|LoadError::ParseError)
    )
}   