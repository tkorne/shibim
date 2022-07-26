extern crate shibim_core as core;
use std::path::Path;
fn main(){
    
    match core::files::process_shb_dir(Path::new("shb")){
        Err(e) =>{
            println!("{}",e);
        }
        Ok(uk) =>{
            for (path,song) in &uk.songs{
                println!("{:?} {}",path,song.name);
                std::fs::write(std::path::Path::new("test").join(path),core::html::Song{
                    song : &song
                }.to_string());
            }

        }
    }
}

