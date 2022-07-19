use std::fs as fs;
use std::path::{Path};
use crate::error::{LoadError, LoadErrorPayload};
use shibim_base::*;
use shibim_parse::parse_song;

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