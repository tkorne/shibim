extern crate shibim_parse;
extern crate shibim_base;
extern crate shibim_core as core;
use std::path::Path;
use shibim_base::*;
fn main(){
    /*/
    match core::files::process_shb_dir(Path::new("shb")){
        Err(e) =>{
            println!("{}",e);
        }
        Ok(uk) =>{
            println!("{:?}",uk.errors);
        }
    } */
    let mut u = core::parser::SHBParser::default();
    u.parse_str(
r#"
name : THE NAME
tonic : C

@E1 The section title
Ya |C·ye yi yo |lines
---
A| some lyrics |G·on new subsection
@E2 Some other section
|Dm·`G· |Dm ·`G·
The other |line |C·is ^chord only
This is ^lyric only
"#);
    println!("{:#?}",u.extract());
    println!("{:?}",core::parser::parse_chord("5+7,2Dm7b5add2#11/G#potato"));
}