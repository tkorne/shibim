
use std::collections::HashMap;
use std::collections::HashSet;

pub type NoteHeight = u8;

pub const CHAR_TONIC_VALUES:[u8;7] = [9,11,0,2,4,5,7];
pub const SHARP_TONIC_NAMES:[&str;12] = ["C","C","D","D","E","F","F","G","G","A","A","B"];
pub const FLAT_TONIC_NAMES:[&str;12] = ["C","D","D","E","E","F","G","G","A","A","B","B"];


#[derive(Debug,Clone,Copy)]
pub enum TonicKind{
    Minor,
    Major,
    Undefined
}
#[derive(Debug,Clone,Copy)]
pub enum ChordKind{
    Minor,
    Major,
    Undefined
}
#[derive(Debug)]
pub enum FormatSize{
    Smaller,
    Small,
    Normal
}


#[derive(Debug,Clone)]
pub struct Song{
    pub name: String,
    pub tonic : NoteHeight,
    pub tonic_kind : TonicKind,
    pub bpm : Option<f32>,
    pub sections : Vec<Section>,
    pub categories : HashSet<String>,
    pub metadata : HashMap<String,String>,
    pub section_names : HashMap<String,usize>,
    pub orders : HashMap<String,Vec<usize>>
}

#[derive(Debug,Clone)]
pub struct CompiledSong{
    pub name: String,
    pub tonic : NoteHeight,
    pub tonic_kind : TonicKind,
    pub bpm : Option<f32>,
    pub sections : Vec<Section>
}

pub struct SongRef<'i> {
    pub name : &'i str,
    pub tonic : NoteHeight,
    pub tonic_kind : TonicKind,
    pub bpm : Option<f32>,
    pub sections : &'i Vec<Section>
}
#[derive(Debug,Clone)]
pub struct SectionName{
    pub kind : String,
    pub number : u16,
    pub version : String,
}

#[derive(Debug,Clone)]
pub struct Section{
    pub name : String,
    pub description : String,
    pub delta_tonic : NoteHeight,
    pub subsections : Vec<Subsection>
    //pub metadata : HashMap<String,String>
}

#[derive(Debug,Clone)]
pub struct Subsection{
    pub metadata : HashMap<String,String>,
    pub lines : Vec<Line>
}

//Line: Vector of possibly empty measures
//Measure: Vector of blocks
//Block: Vector of events (or a tuple of vectors)
type MixedEventList = (Vec<MusicEvent>,Vec<LyricEvent>);

#[derive(Debug,Clone)]
pub enum Line{
    Lyrics  (Vec< Option< Vec<  Vec<LyricEvent> >>>),
    Chords  (Vec< Option< Vec<  Vec<MusicEvent> >>>),
    Mixed   (Vec< Option< Vec< MixedEventList >>>)
}
#[derive(Debug,Clone)]
pub enum LyricEvent{
    LyricText(String),
    LyricBreak
}

#[derive(Debug,Clone)]
pub enum MusicEvent{
    ChordEvent(ChordEvent),
    RepeatMeasure,
    StartRepeat,
    EndRepeat,
    OpenParen,
    CloseParen,
    NumberedMeasure(u16),
    Annotation(String),
    MelodyEvent(Vec<NoteHeight>)
}

#[derive(Debug,Clone)]
pub struct ChordEvent{
    pub root : NoteHeight,
    pub bass : Option<NoteHeight>,
    pub kind : ChordKind,
    pub modifier : Vec<ChordModifier>,
    pub time : Option<TimeOffset>
}

#[derive(Debug,Clone,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub enum ChordKeyword{
    Sus2,
    Sus4,
    Add2,
    Add4,
    Add9,
    Add11,
    Maj,
    K6,
    K5,
    K7,
    K9,
    K11,
    K13,
    K69,
    Aug,
    Dim
}

#[derive(Debug,Clone)]
pub enum ChordAlterationKind{
    Flat,
    Sharp,
    No
}

#[derive(Debug,Clone)]
pub struct ChordAlteration{
    pub kind : ChordAlterationKind,
    pub degree : u8
}

#[derive(Debug,Clone)]
pub enum ChordModifier{
    Keyword(ChordKeyword),
    Alteration(ChordAlteration)
}

#[derive(Debug,Clone)]
pub struct TimeOffset{
    pub beat: i8,
    pub num : u8,
    pub den : u8
}

#[derive(Debug)]
pub struct SonglistEntry{
    pub id_file : String,
    pub rename : Option<String>,
    pub tonic : Option<(u8,bool)>,
    pub explicit_order : Option<Vec<String>>,
    pub named_order : Option<String>,
    pub joined : bool,
    pub inline_data : Option<String>,
    pub file_line : usize //For error reporting
}

impl std::convert::From<Song> for CompiledSong{
    fn from(item: Song) -> Self{
        CompiledSong{
            name : item.name,
            tonic : item.tonic,
            tonic_kind : item.tonic_kind,
            bpm : item.bpm,
            sections : item.sections
        }
    }
}
impl std::convert::From<&Song> for CompiledSong{
    fn from(item: &Song) -> Self{
        CompiledSong{
            name : item.name.clone(),
            tonic : item.tonic,
            tonic_kind : item.tonic_kind,
            bpm : item.bpm,
            sections : item.sections.clone()
        }
    }
}


impl<'i> std::convert::From<&'i Song> for SongRef<'i>{
    fn from(item: &'i Song) -> Self{
        SongRef{
            name : &item.name,
            tonic : item.tonic,
            tonic_kind : item.tonic_kind,
            bpm : item.bpm,
            sections : &item.sections
        }
    }
}

impl<'i> std::convert::From<&'i CompiledSong> for SongRef<'i>{
    fn from(item: &'i CompiledSong) -> Self{
        SongRef{
            name : &item.name,
            tonic : item.tonic,
            tonic_kind : item.tonic_kind,
            bpm : item.bpm,
            sections : &item.sections
        }
    }
}
#[derive(Debug)]
pub struct  SHBParseError{
    pub loc : std::ops::Range<usize>,
    pub msg : String
}

pub enum ParseSongWarnings{
    RepeatedSectionName,
    UnNamed,
    NoTonic
}

pub enum ParseListWarnings{
    SongNotFound(String),
    SongSectionsNotFound(Vec<String>),
    FirstJoined,
    UnknownSongArgs(String)
}
/*
impl std::convert::From<&Song> for SongRef{
    fn from(item: &Song) -> Self{
        SongRef{
            name : item.name.clone(),
            tonic : item.tonic,
            tonic_kind : item.tonic_kind,
            bpm : item.bpm,
            sections : item.sections.clone()
        }
    }
}*/