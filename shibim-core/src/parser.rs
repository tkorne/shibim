use std::borrow::Borrow;
use shibim_base::*;

#[derive(PartialEq,Eq)]
enum ParsingState{
    MetaStart,
    MetaArg,
    MetaVal,
    SectionStart,
    SectionHeadId,
    SectionHeadDesc,
    SubsectionStart,
    LineStart,
    MeasureStart,
    BlockStart,
    MaybeLyricBlock,
    TrueLyricBlock,
    LineEnd,
    SubsectionDelim(u8),
    SubsectionMetaDelim(u8)
}
#[derive(PartialEq,Eq)]
enum ParserStatus{
    New,
    Processing,
    Completed,
    Error
}
struct Parser{
    line : usize,
    line_has_content : bool,
    meta_arg_buffer : String,
    meta_val_buffer : String,
    section_id_buffer : String,
    section_desc_buffer : String,
    lyric_buffer :String,
    chord_buffer : String,
    line_buffer : Vec<Vec<(Vec<MusicEvent>,Vec<LyricEvent>)>>,
    state : ParsingState,
    status : ParserStatus,
    song : Song
}

impl Default for Parser {
    fn default() -> Self {
        Parser{
            line : 0,
            line_has_content : false,
            meta_arg_buffer : String::new(),
            meta_val_buffer : String::new(),
            section_id_buffer : String::new(),
            section_desc_buffer : String::new(),
            line_buffer : Vec::new(),
            lyric_buffer : String::new(),
            chord_buffer : String::new(),
            state : ParsingState::MetaStart,
            status : ParserStatus::New,
            song : Song::default()
        }
    }
}
macro_rules! consume {
    ($sel:expr) => {
        {
            let u = $sel.clone();
            $sel.clear();
            u
        }
    };
}

macro_rules! consume_str {
    ($sel:expr,$s:expr) => {
        {
            let u = $s.to_owned();
            $sel.clear();
            u
        }
    };
}

macro_rules! last_section {
    ($sel:expr) => {
        $sel.song.sections.last_mut().unwrap()
    };
}

macro_rules! last_subsection {
    ($sel:expr) => {
        last_section!($sel).subsections.last_mut().unwrap()
    };
}

macro_rules! buffer_measure {
    ($sel:expr) => {
        $sel.line_buffer.last_mut().unwrap()
    };
}

macro_rules! buffer_block {
    ($sel:expr) => {
        buffer_measure!($sel).last_mut().unwrap()
    };
}


impl Parser{
    fn parse_char(&mut self,c : char){
        if let ParserStatus::New = self.status{
            self.status = ParserStatus::Processing;
        }
        use ParsingState::*;
        let mut retry = true;

        if c == '\r'{
            return; //Do nothing   
        }
        
        while retry {
            retry = false;
            match self.state{
                MetaStart => match c{
                    _ if c.is_whitespace()=>{
                        //Do nothing
                    },
                    '@' =>{
                        self.state = SectionStart;
                    }
                    '_' => {
                        self.state = MetaArg;
                        retry = true;
                        continue;
                    },
                    _ if c.is_alphanumeric() => {
                        self.state = MetaArg;
                        retry = true;
                        continue;
                    },
                    _ => {
                        println!("Unexpected {}",c);
                    }
                },
                MetaArg => match c {
                    ':' =>{
                        self.state = MetaVal;
                    }
                    ' ' | '_' =>{
                        self.meta_arg_buffer.push(' ');
                    }
                    _ if c.is_alphanumeric() => {
                        self.meta_arg_buffer.push(c);
                    }
                    _=>{
                        eprintln!("Unexpected character {}",c)
                    }
                },
                MetaVal => match c {
                    '\n' =>{
                        let trim_arg = self.meta_arg_buffer.trim();
                        if trim_arg.is_empty(){
                            eprintln!("Empty metadata argument");
                        }
                        let trim_val = self.meta_val_buffer.trim();
                        if trim_val.is_empty(){
                            eprintln!("Empty metadata argument");
                        }
                        self.song.metadata.insert(
                            consume_str!(self.meta_arg_buffer,trim_arg),
                            consume_str!(self.meta_val_buffer,trim_val)
                        );
                        self.state = MetaStart;
                    }
                
                    _=>{
                        self.meta_arg_buffer.push(c);

                    }
                }

                SectionStart => {
                    self.song.sections.push(Section::default());
                    self.state = SectionHeadId; 
                    retry = true;
                    continue;
                }

                SectionHeadId => match c {
                    '\n' => {
                        let id_trim = self.section_id_buffer.trim();
                        if id_trim.is_empty(){
                            eprintln!("Empty section name")
                        }
                        last_section!(self).name =
                            consume_str!(self.section_id_buffer,id_trim);
                        self.state = LineStart;
                    }
                    _ if c.is_whitespace() => {
                        let id_trim = self.section_id_buffer.trim();
                        if id_trim.is_empty(){
                            eprintln!("Empty section name")
                        }
                        last_section!(self).name =
                            consume_str!(self.section_id_buffer,id_trim);
                        self.state = SectionHeadDesc;
                    }
                    _ if c.is_alphanumeric() => {
                        self.section_id_buffer.push(c);
                    }
                    '~' =>{
                        self.section_id_buffer.push(c);
                    }
                    _ =>{
                        eprintln!("unexpected {}",c);
                    }
                }
                
                SectionHeadDesc => match c{
                    '\n' => {
                        let desc_trim = self.section_desc_buffer.trim();
                        last_section!(self).name =
                            consume_str!(self.section_desc_buffer,desc_trim);
                        self.state = LineStart;
                    }

                    _ => {
                        self.section_desc_buffer.push(c)
                    }
                }

                SubsectionStart =>{
                    last_section!(self).subsections.push(Subsection::default());
                    self.state = LineStart;
                    retry = true;
                    continue;
                }

                LineStart => match c{
                    '@' => {
                        self.state = SectionStart;
                        self.lyric_buffer.clear();
                    }
                    ' ' => {
                        self.lyric_buffer.push(' ');
                    }
                    '-' => {
                        self.lyric_buffer.push('-');
                        self.state = SubsectionDelim(1);
                    }
                    _=>{
                        self.state = MeasureStart;
                        retry = true;
                        continue;
                    }
                }

                MeasureStart =>{
                    self.state = BlockStart;
                    self.line_buffer.push(Vec::new());
                    retry = true;
                    continue;
                }

                BlockStart =>{
                    self.state = MaybeLyricBlock;
                    buffer_measure!(self).push((Vec::new(),Vec::new()));
                    retry = true;
                    continue;
                }

                MaybeLyricBlock => match c{
                    '\n' => {
                        buffer_block!(self).1.push(
                            LyricEvent::LyricText(consume!(self.lyric_buffer))
                        );
                        self.state = LineEnd;
                        retry = true;
                        continue;
                    }
                    '·'| '*' =>{
                        std::mem::swap(&mut self.lyric_buffer, &mut self.chord_buffer);
                        self.state = TrueLyricBlock;
                    }
                    '|' => {
                        self.state = MeasureStart;
                    }
                    '´' | '`' =>{
                        self.state = BlockStart;
                    }
                    _=>{
                        self.lyric_buffer.push(c);
                    }
                }

                TrueLyricBlock => match c{
                    '|' => {
                        self.state = MeasureStart;
                    }
                    '´' | '`' =>{
                        self.state = BlockStart;
                    }
                    '·' | '*' =>{
                        eprintln!("Error, repeated · or *");
                    }
                    '\n' => {
                        buffer_block!(self).1.push(
                            LyricEvent::LyricText(consume!(self.lyric_buffer))
                        );
                        buffer_block!(self).0.push(
                            MusicEvent::CloseParen //TODO: STUB
                        );
                        self.state = LineEnd;
                        retry = true;
                        continue;
                    }
                    _=>{
                        self.lyric_buffer.push(c);
                    }
                }

                LineEnd => {
                    self.state = LineStart;
                    let mut has_chords = false;
                    let mut has_lyrics = false;
                    for (chord_block,lyric_block)  in self.line_buffer.iter().flatten(){
                        for lyric_fragment in lyric_block{
                            if let LyricEvent::LyricText(text) = lyric_fragment{
                                if !text.trim().is_empty(){
                                    has_chords = true;
                                }
                            }
                        }
                        if !chord_block.is_empty(){
                            has_chords = true;
                        }
                    }
                    match (has_chords,has_lyrics) {
                        (true,true) =>{
                            last_subsection!(self).lines.push(
                                Line::Mixed(consume!(self.line_buffer))
                            );
                        }
                        (true,false) =>{
                            let u = self.line_buffer.iter().map(|measure|{
                                measure.iter().map(|(chords,_)|chords.clone()).collect()
                            }).collect();
                            
                            last_subsection!(self).lines.push(
                                Line::Chords(u)
                            )
                        }
                        (false, _) => {
                            let u = self.line_buffer.iter().map(|measure|{
                                measure.iter().map(|(_,lyrics)|lyrics.clone()).collect()
                            }).collect();
                            
                            last_subsection!(self).lines.push(
                                Line::Lyrics(u)
                            )
                        }
                    }
                }
                SubsectionDelim(1) => match c {
                    '-' => {
                        self.state = SubsectionDelim(2);
                        self.lyric_buffer.push('-');
                    }
                    _ => {
                        self.state = MeasureStart;
                        retry = true;
                        continue;
                    }
                }
                SubsectionDelim(2) => match c{
                    '-' => {
                        self.state = SubsectionDelim(3);
                        self.lyric_buffer.clear();
                    }
                    _ =>{
                        self.state = MeasureStart;
                        retry = true;
                        continue;
                    }
                }
                SubsectionDelim(3) => match c {
                    '\n' => {
                        self.state = SubsectionStart;
                    }
                    '-' => {}
                    _ if c.is_whitespace() => {}
                    _ =>{
                        eprint!("Unexpected {}",c)
                    }
                }
                _ => {}
            }
        }
        if c == '\n'{
            self.line += 1;
            self.line_has_content = false;
        }else if !c.is_whitespace(){
            self.line_has_content = true;
        }
    }
}

fn is_measure_empty(measure : &Vec<(&Vec<MusicEvent>,&Vec<LyricEvent>)>){

}