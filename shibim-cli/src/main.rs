extern crate shibim_parse;
extern crate shibim_base;
extern crate shibim_core as core;
use shibim_base::*;
fn main(){
    let c = shibim_base::ChordEvent{
        root : 1,
        kind : ChordKind::Minor,
        bass : Some(2),
        modifier : vec!(
            ChordModifier::Keyword(
                ChordKeyword::Add11
            ),
            ChordModifier::Keyword(
                ChordKeyword::K7
            )
        ),
        time : None
    };
    let u = core::html::ChordEvent{
        chord : &c,
        use_flats : false
    };
    println!("{}",u);
    
}