extern crate chumsky;
extern crate shibim_base;
use chumsky::prelude::*;
use chumsky::error::*;
use shibim_base::*;
use std::collections::HashMap;
use std::collections::HashSet;

//The thing that makes rustc choke :c

//TODO: box parser into some interface
pub fn parse_song(s: &str,session : &mut SongSessionInfo) -> std::result::Result<shibim_base::Song, Vec<SHBParseError>>{
        //Maybe some more efficient way
    let vec_to_string = |s : Vec<char>|s.into_iter().collect::<String>().trim().to_owned();
    let ident = 
        filter::<_,_,Simple<char>>(|c :&char|c.is_alphanumeric() || *c == '_')
        .repeated().at_least(1)
        .padded_by(just(' ').repeated())
        .map(vec_to_string);
    let meta_arg =
        none_of::<_,_,Simple<char>>("\n|\r")
        .repeated()
        .collect::<String>()
        .map(|x|x.trim().to_owned());
    let meta_arg_yesbar =
        none_of::<_,_,Simple<char>>("\n\r")
        .repeated()
        .collect::<String>()
        .map(|x|x.trim().to_owned());
    let meta_line =
        ident.then_ignore(just(':'))
        .map(|x|x.to_lowercase())
        .then(meta_arg.clone());
    let meta_line_yesbar =
        ident.then_ignore(just(':'))
        .map(|x|x.to_lowercase())
        .then(meta_arg_yesbar.clone());
    
    let meta_section =
        meta_line_yesbar.clone()
        .then_ignore(one_of("\n|\r").repeated())
        .repeated();
    
    
    let section_name_noparse = 
        filter::<_,_,Simple<char>>(|c : &char|c.is_alphabetic())
        .repeated().at_least(1)
        .chain::<char,Vec<char>,_>(
            filter(|x : &char|x.is_digit(10)).repeated().at_most(3)
        ).chain::<char,Vec<char>,_>(
            filter(|x: &char|x.is_alphanumeric() || *x == '~')
            .repeated()
        ).collect::<String>();
    let section_order =
        ident
        .then(
            section_name_noparse.padded()
            .repeated().at_least(1)
        )
        .padded();
        /*
    let section_name = 
        filter::<_,_,Simple<char>>(|c : &char|c.is_alphabetic())
        .repeated().at_least(1).collect::<String>()
        .then(
            filter(|x : &char|x.is_digit(10)).repeated().at_most(3)
            .collect::<String>()
            .map(|x|x.parse::<u16>().unwrap())
        ).then(
            filter(|x: &char|x.is_alphanumeric())
            .repeated().collect::<String>()
        ).map(|((k,n),v)| SectionName{kind:k,number:n,version:v});*/
    
    let section_header = 
        just::<_,_,Simple<char>>('@')
        .ignore_then(section_name_noparse)
        .then(meta_arg);
    
    //the first text of a line can't have --
    let first_lyric_text = 
        just::<_,_,Simple<char>>('^').map(|_|LyricEvent::LyricBreak).or(
            none_of("@|^\n\r·*`´-")
            .chain(
                none_of("|^\n\r·*`´")
                .repeated()
            )
            .collect::<String>()
            .map(LyricEvent::LyricText)
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
        just::<_,_,Simple<char>>('^').map(|_|LyricEvent::LyricBreak).or(
            none_of("|^\n\r·*`´")
            .repeated().at_least(1)
            .collect::<String>()
            .map(LyricEvent::LyricText)
        ).repeated().at_least(1);
    //TODO: Check ranges
    let short_uint = text::digits::<_,Simple<char>>(10).map(|x|x.parse::<u8>().unwrap_or(0));
    let short_int = text::digits::<_,Simple<char>>(10).map(|x|x.parse::<i8>().unwrap_or(0));
    let time_fraction = 
        just::<_,_,Simple<char>>('+')
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
                    den,
                    num
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
        one_of::<_,_,Simple<char>>("ABCDEFG").map(|c| CHAR_TONIC_VALUES [((c as u32 -'A' as u32) % 7) as usize])
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
    //Tonality is handled differently to chords
    //When it is minor, the root corresponds to the 
    //relative major (ie Am = C = 0)
    let tonality =
        chord_root.clone()
        .then_with(|root|just('m').or_not().map(move |x|match x{
            Some(_)=> ((root+3)%12,TonicKind::Minor),
            _ => (root,TonicKind::Major)
        }));
    let keyword = just::<char,_,Simple<char>>;
    let chord_kind_alteration = choice::<_,Simple<char>>((
        keyword("no").to(ChordAlterationKind::No),
        keyword("b").to(ChordAlterationKind::Flat),
        keyword("#").to(ChordAlterationKind::Sharp)
    ));
    let chord_num_alteration = choice::<_,Simple<char>>((
        keyword("13").to(13),
        keyword("11").to(11),
        filter::<_,_,Simple<char>>(|c:&char|c.is_digit(10))
            .map(|x| (x as u32 - '0' as u32)as u8)
    ));
    let chord_keyword = 
        choice::<_,Simple<char>>( (
            keyword("sus4").to(ChordKeyword::Sus4),
            keyword("sus2").to(ChordKeyword::Sus2),
            keyword("add4").to(ChordKeyword::Add4),
            keyword("add2").to(ChordKeyword::Add2),
            keyword("add9").to(ChordKeyword::Add9),
            keyword("add11").to(ChordKeyword::Add11),
            keyword("Maj").to(ChordKeyword::Maj),
            keyword("6/9").to(ChordKeyword::K69),
            keyword("aug").to(ChordKeyword::Aug),
            keyword("dim").to(ChordKeyword::Dim),
            keyword("6").to(ChordKeyword::K6),
            keyword("5").to(ChordKeyword::K5),
            keyword("7").to(ChordKeyword::K7),
            keyword("9").to(ChordKeyword::K9),
            keyword("11").to(ChordKeyword::K11),
            keyword("13").to(ChordKeyword::K13),
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
            root,
            kind: 
                if min.is_some(){
                    ChordKind::Minor
                }else{
                    ChordKind::Major
                },
            modifiers : mods,
            bass,
            time
        });
    
    let music_event = choice::<_,Simple<char>>((
        chord.clone().map(MusicEvent::ChordEvent),

        simple_melody.map(MusicEvent::MelodyEvent),

        keyword("%").to(MusicEvent::RepeatMeasure),

        none_of::<_,_,Simple<char>>("\"\n").repeated()
            .delimited_by(just("\""),just("\""))
            .collect::<String>().map(MusicEvent::Annotation),
        
        keyword(":-").to(MusicEvent::StartRepeat),
        
        keyword("-:").to(MusicEvent::EndRepeat),
        
        keyword("(").to(MusicEvent::OpenParen),
        
        keyword(")").to(MusicEvent::CloseParen),
        
        text::int::<_,Simple<char>>(10)
            .map(|x|MusicEvent::NumberedMeasure(x.parse::<u16>().unwrap())) //TODO: bounds
    ));

    let chord_text =
        music_event.then_ignore(just(' ').repeated()).repeated()
        .then_ignore(one_of("·*"));
    let first_block =
        chord_text.clone().or_not().then(first_lyric_text.clone().map(Some))
        .or(
            chord_text.clone().map(Some).then(first_lyric_text.clone().or_not())
        );
    let block =
        chord_text.clone().or_not().then(lyric_text.clone().map(Some))
        .or(
            chord_text.clone().map(Some).then(lyric_text.or_not())
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
        for imeasure in arr.iter().flatten(){
                for (ichords,ilyrics) in imeasure{
                    if ichords.is_some(){
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
            match (has_chords,has_lyrics){
                (true,true) => {
                    #[allow(clippy::manual_map)] //The None's are there if one needs to change space behaviour
                    Line::Mixed(
                        arr.drain(0..).map(|imeasure| match imeasure{
                            None => Vec::new(),
                            Some(mut x) => x.drain(0..).map(|(ichords,ilyrics)|{
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
                            }).collect::<Vec<(Vec<MusicEvent>,Vec<LyricEvent>)> >()
                        }).collect::<Vec<_>>()
                    )
                }
                (true,false)=>{
                    #[allow(clippy::manual_map)] 
                    Line::Chords(
                        arr.drain(0..).map(|imeasure| match imeasure{
                            None => Vec::new(),
                            Some(mut ameasure) => ameasure.drain(0..).map(
                                |(ichords,_lyrics)|ichords.unwrap_or_default()
                            ).collect::<Vec<Vec<MusicEvent>> >()
                        }).collect::<Vec<_>>()
                    )
                },
                (false,true)|(false,false)=>{
                    #[allow(clippy::manual_map)] 
                    Line::Lyrics(
                        arr.drain(0..).map(|imeasure| match imeasure{
                            None => Vec::new(),
                            Some(mut x) => x.drain(0..).map(
                                |(_chords,ilyrics)|ilyrics.unwrap()
                            ).collect::<Vec<Vec<LyricEvent>> >()
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
        ).then_ignore(end())
        .map(|(mut meta,secs)|{
            let mut cats = HashSet::new();
            let mut nmeta = HashMap::new();
            let name = String::new();
            let mut full_order = Vec::new();
            let mut bpm = None;
            let mut section_names : HashMap<String,usize> = HashMap::new();
            let mut orders :  HashMap<String,Vec<usize>> = HashMap::new();
            for (i, section) in secs.iter().enumerate(){
                if let Some(previous) = section_names.get(&section.name){
                    if section.subsections.is_empty(){
                        full_order.push(*previous);
                    }else{
                        session.emit_warning(ParseSongWarnings::RepeatedSectionName(section.name.clone()));
                    }
                }else{
                    section_names.insert(section.name.clone(), i);
                    full_order.push(i);
                }

            }
            let mut tonic = 0;
            let mut tonic_kind = TonicKind::Undefined;
            for (var,val) in meta.drain(..){
                match var.as_str(){
                    "cat"|"category"|"categories" =>{
                        cats.insert(val);
                    }
                    "tonic"|"tone"|"root" =>{
                        if let Ok((atonic,akind)) = tonality.clone().padded().parse(val){
                            tonic = atonic;
                            tonic_kind = akind;
                        }else{
                            session.emit_warning(ParseSongWarnings::WrongTonicFormat);
                        }
                    }
                    "ord" | "order" =>{
                        if let Ok((order_name,srefs)) = section_order.parse(val){
                            let norder = srefs.iter().filter_map(|x|{
                                if let Some(u) = section_names.get(x) {
                                    Some(*u)
                                }else{
                                    session.emit_warning(ParseSongWarnings::SectionNotFound(x.clone()));
                                    None
                                }
                                
                            }).collect::<Vec<usize>>();
                            orders.insert(order_name,norder);
                        }
                    }
                    "bpm" =>{
                        if let Ok(abpm) = val.trim().parse::<f32>(){
                            bpm = Some(abpm);
                        }else{
                            eprintln!("Error parsing BPM");
                        }
                    }
                    _ => {
                        nmeta.insert(var, val);
                    }
                }
            }
            Song{
                name ,
                tonic ,
                tonic_kind,
                bpm ,
                sections : secs,
                categories : cats,
                metadata : nmeta,
                section_names ,
                orders 
            }
        });

    song.parse(s).map_err(|errvec| 
        errvec.iter().map(|e|{
            println!("{:?}",e);
            println!("{:?}",&s[e.span().start..]);
            println!("{:?}",s);
            SHBParseError{
                loc : e.span(),
                msg : s[e.span()].to_owned()
            }
        }).collect()
    )
}