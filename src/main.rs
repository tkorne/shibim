
use std::collections::HashMap;
use std::collections::HashSet;

pub type NoteHeight = u8;

const CHAR_TONIC_VALUES:[u8;7] = [9,11,0,2,4,5,7];


#[derive(Debug,Clone)]
pub enum TonicKind{
    Minor,
    Major,
    Undefined
}
#[derive(Debug,Clone)]
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
pub struct SectionName{
    pub kind : String,
    pub number : u16,
    pub version : String,
}

#[derive(Debug,Clone)]
pub struct Section{
    pub name : SectionName,
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
#[derive(Debug,Clone)]
pub enum Line{
    Lyrics  (Vec< Option< Vec<  Vec<LyricEvent> >>>),
    Chords  (Vec< Option< Vec<  Vec<MusicEvent> >>>),
    Mixed   (Vec< Option< Vec< (Vec<MusicEvent>,Vec<LyricEvent>) >>>)
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
    NumberedMeasure(u16),
    Annotation(String),
    MelodyEvent(Vec<NoteHeight>)
}

#[derive(Debug,Clone)]
pub struct ChordEvent{
    root : NoteHeight,
    bass : Option<NoteHeight>,
    kind : ChordKind,
    modifier : Vec<ChordModifier>,
    time : Option<TimeOffset>
}

#[derive(Debug,Clone,PartialEq)]
pub enum ChordKeywords{
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
    kind : ChordAlterationKind,
    degree : u8
}

#[derive(Debug,Clone)]
pub enum ChordModifier{
    Keyword(ChordKeywords),
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

/*
        let space_ident = 
            filter::<_,_,Cheap<char>>(|c: &char| c.is_whitespace() ).repeated().ignore_then(
            filter::<_,_,Cheap<char>>(|c: &char| c.is_alphanumeric() || *c == ' ').repeated().at_least(1)
            ).map(vec_to_string);
            */
mod parser{
    extern crate chumsky;
    use chumsky::prelude::*;
    use chumsky::error::*;
    use super::*;
    pub fn parse_identifier(s: &str){
         //Maybe some more efficient way
        let vec_to_string = |s : Vec<char>|s.into_iter().collect::<String>().trim().to_owned();
        let ident = 
            filter::<_,_,Cheap<char>>(|c :&char|c.is_alphanumeric())
            .repeated().at_least(1)
            .padded_by(just(' ').repeated())
            .map(vec_to_string);
        let meta_arg =
            none_of::<_,_,Cheap<char>>("\n|\r")
            .repeated()
            .collect::<String>()
            .map(|x|x.trim().to_owned());
        let meta_arg_yesbar =
            none_of::<_,_,Cheap<char>>("\n\r")
            .repeated()
            .collect::<String>()
            .map(|x|x.trim().to_owned());
        let meta_line =
            ident.clone().then_ignore(just(':'))
            .map(|x|x.to_lowercase())
            .then(meta_arg.clone());
        let meta_line_yesbar =
            ident.clone().then_ignore(just(':'))
            .map(|x|x.to_lowercase())
            .then(meta_arg_yesbar.clone());
        
        let meta_section =
            meta_line_yesbar.clone()
            .then_ignore(one_of("\n|\r").repeated())
            .repeated()
            .map(|arr|{
                let mut hashmap = HashMap::new();
                for (key,value) in arr{
                    hashmap.insert(key, value);
                }
                hashmap
            });
        
        let section_name = 
            filter::<_,_,Cheap<char>>(|c : &char|c.is_alphabetic())
            .repeated().at_least(1).collect::<String>()
            .then(
                filter(|x : &char|x.is_digit(10)).repeated().at_most(3)
                .collect::<String>()
                .map(|x|x.parse::<u16>().unwrap())
            ).then(
                filter(|x: &char|x.is_alphanumeric())
                .repeated().collect::<String>()
            ).map(|((k,n),v)| SectionName{kind:k,number:n,version:v});
        
        let section_header = 
            just::<_,_,Cheap<char>>('@')
            .ignore_then(section_name)
            .then(meta_arg);
        
        //the first text of a line can't have --
        let first_lyric_text = 
            just::<_,_,Cheap<char>>('^').map(|_|LyricEvent::LyricBreak).or(
                none_of("@|^\n\r·*`´-")
                .chain(
                    none_of("|^\n\r·*`´")
                    .repeated()
                )
                .collect::<String>()
                .map(|x|LyricEvent::LyricText(x))
            ).or(
                just('-')
                .chain(
                    none_of("|^\n\r·*`´-")
                )
                .then(
                    none_of("|^\n\r·*`´")
                    .repeated()
                )
                .map(|(mut x,mut y)|LyricEvent::LyricText(x.drain(0..).chain(y.drain(0..)).collect::<String>() ))
            ).repeated().at_least(1);
        let lyric_text =
            just::<_,_,Cheap<char>>('^').map(|_|LyricEvent::LyricBreak).or(
                none_of("|^\n\r·*`´")
                .repeated().at_least(1)
                .collect::<String>()
                .map(|x|LyricEvent::LyricText(x))
            ).repeated().at_least(1);
        //TODO: Check ranges
        let short_uint = text::digits::<_,Cheap<char>>(10).map(|x|x.parse::<u8>().unwrap_or(0));
        let short_int = text::digits::<_,Cheap<char>>(10).map(|x|x.parse::<i8>().unwrap_or(0));
        let time_fraction = 
            just::<_,_,Cheap<char>>('+')
            .ignore_then(short_uint)
            .then_ignore(just(','))
            .then(short_uint)
            .or(
                one_of("'\"").to((1,2))
            );
        let time_annoation = 
            (
                short_int.or(
                    just('<').to(-1))
                )
                .then(time_fraction.or_not()
            )
            .map(|(beat,frac)|
                if let Some((num,den)) = frac{
                    TimeOffset{
                        beat : beat as i8,
                        den : den,
                        num : num
                    }
                }else{
                    TimeOffset{
                        beat : beat as i8,
                        den : 1,
                        num : 0
                    }
                }
            );
        //TODO?: Double accidentals are not supported
        let chord_root =
            one_of::<_,_,Cheap<char>>("ABCDEFG").map(|c| CHAR_TONIC_VALUES [((c as u32 -'A' as u32) % 7) as usize])
            .then(one_of("#b").or_not())
            .map(|(root,alt)|
                if let Some(alt) = alt {
                    if alt == '#'{
                        root+1
                    }else{
                        root-1
                    }
                }else{
                    root
                }
            );
        let keyword = just::<char,_,Cheap<char>>;
        let chord_kind_alteration = choice::<_,Cheap<char>>((
            keyword("no").to(ChordAlterationKind::No),
            keyword("b").to(ChordAlterationKind::Flat),
            keyword("#").to(ChordAlterationKind::Sharp)
        ));
        let chord_num_alteration = choice::<_,Cheap<char>>((
            keyword("13").to(13),
            keyword("11").to(11),
            filter::<_,_,Cheap<char>>(|c:&char|c.is_digit(10))
                .map(|x|x as u8 - '0' as u8)
        ));
        let chord_keyword = 
            choice::<_,Cheap<char>>( (
                keyword("sus4").to(ChordKeywords::Sus4),
                keyword("sus2").to(ChordKeywords::Sus2),
                keyword("add4").to(ChordKeywords::Add4),
                keyword("add2").to(ChordKeywords::Add2),
                keyword("add9").to(ChordKeywords::Add9),
                keyword("add11").to(ChordKeywords::Add11),
                keyword("Maj").to(ChordKeywords::Maj),
                keyword("6/9").to(ChordKeywords::K69),
                keyword("aug").to(ChordKeywords::Aug),
                keyword("dim").to(ChordKeywords::Dim),
                keyword("6").to(ChordKeywords::K6),
                keyword("5").to(ChordKeywords::K5),
                keyword("7").to(ChordKeywords::K7),
                keyword("9").to(ChordKeywords::K9),
                keyword("11").to(ChordKeywords::K11),
                keyword("13").to(ChordKeywords::K13),
            ));
        let chord_alteration =
            chord_kind_alteration
            .then(chord_num_alteration)
            .map(|(k,n)|ChordAlteration{kind:k,degree:n});
        let chord_extensions =
            chord_keyword
            .map(ChordModifier::Keyword).or(
                chord_alteration
                .map(ChordModifier::Alteration)
            ).repeated();
        let simple_melody =
            chord_root.clone()
            .padded_by(just(" ").repeated())
            .repeated().at_least(1)
            .delimited_by(just('['),just(']'));
        let chord =
            time_annoation.or_not()
            .then(chord_root.clone())
            .then(just('m').or_not())
            .then(chord_extensions)
            .then(
                one_of("/\\").ignore_then(
                    chord_root.clone()
                ).or_not()
            ) //(((u8, std::option::Option<char>), std::vec::Vec<ChordModifier>), std::option::Option<u8>)
            .map(|((((time,root),min),mods),bass)|ChordEvent{
                root: root,
                kind: 
                    if let Some(_) = min{
                        ChordKind::Minor
                    }else{
                        ChordKind::Major
                    },
                modifier : mods,
                bass : bass,
                time : time
            });
        
        let music_event = choice::<_,Cheap<char>>((
            chord.clone().map(MusicEvent::ChordEvent),
            simple_melody.map(MusicEvent::MelodyEvent),
            keyword("%").to(MusicEvent::RepeatMeasure),
            none_of::<_,_,Cheap<char>>("\"\n").repeated()
                .delimited_by(just("\""),just("\""))
                .collect::<String>().map(MusicEvent::Annotation),
            keyword(":-").to(MusicEvent::StartRepeat),
            keyword("-:").to(MusicEvent::EndRepeat),
            text::int::<_,Cheap<char>>(10)
                .map(|x|MusicEvent::NumberedMeasure(x.parse::<u16>().unwrap())) //TODO: bounds
        ));

        let chord_text =
            music_event.then_ignore(just(' ').repeated()).repeated()
            .then_ignore(one_of("·*"));
        let first_block =
            chord_text.clone().or_not().then(first_lyric_text.clone().map(|x|Some(x)))
            .or(
                chord_text.clone().map(|x|Some(x)).then(first_lyric_text.clone().or_not())
            );
        let block =
            chord_text.clone().or_not().then(lyric_text.clone().map(|x|Some(x)))
            .or(
                chord_text.clone().map(|x|Some(x)).then(lyric_text.or_not())
            );

        let first_measure =
            first_block.then_ignore(one_of("`´").or_not())
            .chain(
                block.clone().then_ignore(one_of("`´").or_not())
                .repeated()
            );
        let measure = 
            block.clone().then_ignore(one_of("`´").or_not())
            .repeated().at_least(1);
        
        
        
            /*
                just('|').map(|_|None) 
            .or(measure.then_ignore(just('|').or_not()).map(|x|Some(x)))
            .repeated().map(|mut arr|{
            */

        //TODO: Do whitespace only measures are 'empty' measures or lyrics
        //Maybe only for chord-only measures
        let line =
            (
                first_measure.clone().or_not()
                .then_ignore(just('|'))
                .chain(
                    measure.clone().or_not()
                    .separated_by(just('|'))
                    .at_least(1)
                )
            )
            .or(
                first_measure.clone()
                .map(|x|vec!(Some(x)))
            )
            .map(|mut arr|{
                let mut has_chords = false;
                let mut has_lyrics = false;
                println!("{:?}",arr);
                for imeasure in &arr{
                    //Count only non empty measures
                    if let Some(imeasure) = imeasure {
                        for (ichords,ilyrics) in imeasure{
                            if let Some(_) = ichords{
                                has_chords = true;
                            }
                            if let Some(text_arr) = ilyrics{
                                for text_evt in text_arr{
                                    match text_evt{
                                        LyricEvent::LyricBreak =>{},
                                        LyricEvent::LyricText(str) =>{
                                            if !str.trim().is_empty(){
                                                has_lyrics = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            if has_chords && has_lyrics{
                                break;
                            }
                        }
                    }
                }
                match (has_chords,has_lyrics){
                    (true,true) => {
                        Line::Mixed(
                            arr.drain(0..).map(|imeasure| match imeasure{
                                None => None,
                                Some(mut x) => Some(x.drain(0..).map(|(ichords,ilyrics)|{
                                    if let Some(ichords) = ichords {
                                        if let Some(ilyrics) = ilyrics {
                                            (ichords,ilyrics)
                                        }else{
                                            (ichords,Vec::new())
                                        }
                                    }else if let Some(ilyrics) = ilyrics{
                                        (Vec::new(),ilyrics)
                                    }else{
                                        unreachable!();
                                        //(Vec::new(),Vec::new())
                                    }
                                }).collect::<Vec<(Vec<MusicEvent>,Vec<LyricEvent>)> >())
                            }).collect::<Vec<_>>()
                        )
                    }
                    (true,false)=>{
                        Line::Chords(
                            arr.drain(0..).map(|imeasure| match imeasure{
                                None => None,
                                Some(mut x) => Some(x.drain(0..).map(
                                    |(ichords,_lyrics)|ichords.unwrap()
                                ).collect::<Vec<Vec<MusicEvent>> >())
                            }).collect::<Vec<_>>()
                        )
                    },
                    (false,true)|(false,false)=>{
                        Line::Lyrics(
                            arr.drain(0..).map(|imeasure| match imeasure{
                                None => None,
                                Some(mut x) => Some(x.drain(0..).map(
                                    |(_chords,ilyrics)|ilyrics.unwrap()
                                ).collect::<Vec<Vec<LyricEvent>> >())
                            }).collect::<Vec<_>>()
                        )
                    }
                }
            });
        
        let subsection_meta_line =
            just("--!").ignore_then(
                meta_line.clone()
                .then_ignore(just('|').or_not())
                .repeated()
            ).map(|arr|{
                let mut hashmap = HashMap::new();
                for (key,value) in arr{
                    hashmap.insert(key, value);
                }
                hashmap
            });
        
        let subsection = 
            subsection_meta_line.clone().then_ignore(
                one_of("\n\r").repeated()
            ).repeated()
            .map(|mut x|{
                if x.len() == 1{
                    x.drain(0..1).next().unwrap()
                }else{
                    x.drain(0..).fold(HashMap::new(),|mut acc,x|{acc.extend(x);acc}) //TODO: Maybe not very efficient, but it was fun
                }
            }).then(
                line.clone().then_ignore(
                    one_of("\n\r").repeated()
                ).repeated()
            ).map(|(meta,ve)|Subsection{
                metadata : meta,
                lines : ve
            });
        
        let section =
            section_header.clone()
            .then_ignore(
                one_of("\n\r").repeated()
            )
            .then(
                subsection.clone()
                .separated_by(
                    just("--")
                    .then(none_of("\n\r").repeated())
                    .then(one_of("\n\r").repeated())
                )
            ).map(|((n,d),s)|Section{
                name : n,
                description : d,
                delta_tonic : 0,
                subsections : s
            });
        let song = 
            meta_section.padded()
            .then(
                section.clone().repeated()
            );

        println!("{:#?}",song.parse(
r#"
name: A Dios Sea La Gloria |E
tonic: C
@I1 Intro (aprox.)
Am·|%·|Fm·|Gsus4 G13·

@E1 Como agradecer	
|C·Cómo he de expres|Em/B·ar
Lo que |C7/Bb·Dios por mí ha |Asus4 A7·hecho
|Dm·Que sin mer|F·ecer`(C/E?)·
Dio |Dm·su sangre `G7· `carme|C·sí`G7·

@C1 Carísimos Ñ
--!ref: Read the manua | h:ood
--!plz: please.
Now, I present|·Cm
|A little|G·thing`asd`G·`hadoop
||
----
Good morin
----
--!ref:PIZAA!
Hello|G·est'there|sad^a`D·Ds"#
        ));
    }
}

fn main() {
    println!("{:?}",parser::parse_identifier("jacobi9 0"));
}