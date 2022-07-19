use std::collections::HashMap;
use shibim_base::ChordKeyword;
lazy_static::lazy_static! {
    pub static ref EN: HashMap<&'static str, &'static str> = {
        HashMap::from([
            ("Mon", "Mon"),
            ("Tue", "Tue"),
            ("Wed", "Wed"),
            ("Thu", "Thu"),
            ("Fri","Fri"),
            ("Sat","Sat"),
            ("Sun","Sat")
        ])
    };
    pub static ref CHORD_KEYWORDS_NAMES: HashMap<ChordKeyword, &'static str> = {
        HashMap::from([
            (ChordKeyword::Sus2,"sus2"),
            (ChordKeyword::Sus4,"sus4"),
            (ChordKeyword::Add2,"add2"),
            (ChordKeyword::Add4,"add4"),
            (ChordKeyword::Add9,"add9"),
            (ChordKeyword::Add11,"add11"),
            (ChordKeyword::Dim,"dim"),
            (ChordKeyword::Aug,"aug"),
            (ChordKeyword::Maj,"Δ"),
            (ChordKeyword::K6,"6"),
            (ChordKeyword::K7,"7"),
            (ChordKeyword::K9,"9"),
            (ChordKeyword::K11,"11"),
            (ChordKeyword::K13,"13"),
            (ChordKeyword::K69,"6/9"),
        ])
    };
}

