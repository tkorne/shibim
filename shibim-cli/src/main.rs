extern crate shibim_parse;
extern crate shibim_base;
extern crate shibim_core as core;
use std::path::Path;
use shibim_base::*;
fn main(){
    match core::files::process_shb_dir(Path::new("shb")){
        Err(e) =>{
            println!("{}",e);
        }
        Ok(uk) =>{
            println!("{:?}",uk.names);
        }
    }  
}