use std::collections::HashMap;
use std::fs as fs;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use crate::util::Itertools;
use crate::error::{LoadError, LoadErrorPayload, VisitorError};
use shibim_base::*;
use shibim_parse::parse_song;

pub fn read_shb(file_name : &Path) -> 
    Result<Song,LoadError>{
    let mut session = shibim_base::SongSessionInfo{cur_file:Some(file_name.to_owned())};
    let u = fs::read_to_string(file_name);
    let u = u.map_err(|e|
        LoadError{
            file : file_name.to_string_lossy().to_string(),
            detail : LoadErrorPayload::IOError(e)
        }
    );
    u.and_then(|s|
        parse_song(&s,&mut session)
        .map_err(|e|
            LoadError{
                file : file_name.to_string_lossy().to_string(),
                detail : LoadErrorPayload::ParseError(e)
            }
        )
    )
}   



fn get_dir_filelist_ext(dir : &Path,req_ext : &OsStr) -> 
Result<Vec<PathBuf>,std::io::Error>{
    let dir = fs::read_dir(dir)?;
    //TODO: ignore unreadable files like this?
    let u  = dir.filter_map(|entry|{
        let entry = entry.ok()?;
        let path = entry.path();
        if path.extension()? == req_ext{
            Some(path)
        }else{
            None
        }
    });
    Ok(u.collect())
}

pub fn process_shb_dir(dir : &Path)->Result<SHBBatchResults,std::io::Error>{
    let paths = get_dir_filelist_ext(dir, OsStr::new("shb"))?;
    //let mut top_names = HashMap::new();
    let mut names = HashMap::new();
    let (songs,errors) : (Vec<_>,Vec<_>) = 
        paths.iter()
        .map(|f|
            (f.to_owned(),read_shb(f))
        )
        .partition_result(|(u,r)|{
            r.map(|song|(u,song))
        });
    for (i,(name,_song)) in songs.iter().enumerate(){
        if let Ok(stem_name) = name.strip_prefix(dir){
            let topname = 
                stem_name.with_extension("")
                .to_string_lossy().to_string();
            names.insert(topname,i);
        }else{
            unreachable!();//can read_dir return a diferent prefix?
        }
    }
    Ok(SHBBatchResults{
        songs,errors, names
    })
}

pub struct SHBBatchResults{
    pub songs : Vec<(PathBuf,Song)>,
    pub errors : Vec<LoadError>,
    pub names : HashMap<String,usize>
}


trait SongVisitor{
    fn process(&mut self, e : &Song) -> Result<(),VisitorError>;
}