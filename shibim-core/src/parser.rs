use shibim_base::*;

#[derive(PartialEq,Eq,Debug)]
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
    SubsectionMetaArg,
    SubsectionMetaVal
}
#[derive(PartialEq,Eq,Debug)]
enum ParserStatus{
    New,
    Processing,
    Completed,
    Error
}
pub struct SHBParser{
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

impl Default for SHBParser {
    fn default() -> Self {
        SHBParser{
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

macro_rules! parse_lyric_buffer {
    ($self:expr) => {
        {
            let mut fragment_start : usize = 0;
            for (i,c) in $self.lyric_buffer.char_indices(){
                if c == '^'{
                    if i > fragment_start{
                        let slice = &$self.lyric_buffer[fragment_start..i];
                        buffer_block!($self).1.push(
                            LyricEvent::LyricText(
                                slice.to_owned()
                            )
                        );
                        //Caret has length 1
                        fragment_start = i + 1;
                    }
                    buffer_block!($self).1.push(LyricEvent::LyricBreak);
                }
            }
            if fragment_start < $self.lyric_buffer.len(){
                buffer_block!($self).1.push(
                    LyricEvent::LyricText(consume_str!( $self.lyric_buffer, $self.lyric_buffer[fragment_start..]))
                );
            }
        }
    };
}


impl SHBParser{
    pub fn parse_char(&mut self,c : char){
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
            println!("{}→{:?}",c,self.state);
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
                    '\n' =>{
                        eprintln!("Metadata '{}' without assigned value",self.meta_arg_buffer);
                        self.meta_arg_buffer.clear();
                        self.state = MetaStart;
                    }
                    ' ' | '_' =>{
                        self.meta_arg_buffer.push(c);
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
                            eprintln!("Empty metadata name");
                        }
                        let trim_val = self.meta_val_buffer.trim();
                        if trim_val.is_empty(){
                            eprintln!("Empty metadata value");
                        }
                        self.song.metadata.insert(
                            consume_str!(self.meta_arg_buffer,trim_arg),
                            consume_str!(self.meta_val_buffer,trim_val)
                        );
                        self.state = MetaStart;
                    }
                
                    _=>{
                        self.meta_val_buffer.push(c);

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
                        last_section!(self).description =
                            consume_str!(self.section_desc_buffer,desc_trim);
                        self.state = SubsectionStart;
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
                        //Parse delayed string
                        parse_lyric_buffer!(self);

                        self.state = LineEnd;
                        retry = true;
                        continue;
                    }
                    '·'| '*' =>{
                        std::mem::swap(&mut self.lyric_buffer, &mut self.chord_buffer);
                        self.state = TrueLyricBlock;
                    }
                    '|' => {
                        parse_lyric_buffer!(self);
                        self.state = MeasureStart;
                    }
                    '´' | '`' =>{
                        parse_lyric_buffer!(self);
                        self.state = BlockStart;
                    }
                    _=>{
                        self.lyric_buffer.push(c);
                    }
                }

                TrueLyricBlock => match c{
                    '|' => {
                        buffer_block!(self).1.push(
                            LyricEvent::LyricText(consume!(self.lyric_buffer))
                        );
                        buffer_block!(self).0.push(
                            MusicEvent::Annotation(consume!(self.chord_buffer)) //TODO: STUB
                        );
                        self.state = MeasureStart;
                    }
                    '´' | '`' =>{
                        buffer_block!(self).1.push(
                            LyricEvent::LyricText(consume!(self.lyric_buffer))
                        );
                        buffer_block!(self).0.push(
                            MusicEvent::Annotation(consume!(self.chord_buffer)) //TODO: STUB
                        );
                        self.state = BlockStart;
                    }
                    '·' | '*' =>{
                        eprintln!("Error, repeated · or *");
                    }
                    '^' => {
                        buffer_block!(self).1.push(
                            LyricEvent::LyricText(consume!(self.lyric_buffer))
                        );
                        buffer_block!(self).1.push(LyricEvent::LyricBreak);
                    }
                    '\n' => {
                        buffer_block!(self).1.push(
                            LyricEvent::LyricText(consume!(self.lyric_buffer))
                        );
                        buffer_block!(self).0.push(
                            MusicEvent::Annotation(consume!(self.chord_buffer)) //TODO: STUB
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
                    if !self.line_has_content{
                        self.line_buffer.clear();
                        break;
                    }
                    let mut has_chords = false;
                    let mut has_lyrics = false;
                    for (chord_block,lyric_block)  in self.line_buffer.iter().flatten(){
                        for lyric_fragment in lyric_block{
                            if let LyricEvent::LyricText(text) = lyric_fragment{
                                if !text.trim().is_empty(){
                                    has_lyrics = true;
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
                    self.line_buffer.clear();
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
                    '!' => {
                        self.state = SubsectionMetaArg;
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
                SubsectionMetaArg => match c{
                    ':' =>{
                        self.state = SubsectionMetaVal;
                    }
                    ' ' | '_' =>{
                        self.meta_arg_buffer.push(c);
                    }
                    '\n' =>{
                        eprintln!("Subsection metadata '{}' without assigned value",self.meta_arg_buffer);
                        self.meta_arg_buffer.clear();
                        self.state = LineStart;
                    }
                    _ if c.is_alphanumeric() => {
                        self.meta_arg_buffer.push(c);
                    }
                    _=>{
                        eprintln!("Unexpected character {}",c)
                    }
                }
                SubsectionMetaVal => match c {
                    '\n' =>{
                        let trim_arg = self.meta_arg_buffer.trim();
                        if trim_arg.is_empty(){
                            eprintln!("Empty metadata name");
                        }
                        let trim_val = self.meta_val_buffer.trim();
                        if trim_val.is_empty(){
                            eprintln!("Empty metadata value");
                        }
                        last_subsection!(self).metadata.insert(
                            consume_str!(self.meta_arg_buffer,trim_arg),
                            consume_str!(self.meta_val_buffer,trim_val)
                        );
                        self.state = LineStart;
                    }
                
                    _=>{
                        self.meta_val_buffer.push(c);

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
    pub fn parse_str(&mut self, s: &str){
        for c in s.chars(){
            self.parse_char(c);
        }
    }
    pub fn finalize(&mut self){
        self.parse_char('\n');
        if self.status != ParserStatus::Error{
            self.status = ParserStatus::Completed;
        }
    }
    pub fn extract(self)->Song{
        self.song
    }
} 

pub fn parse_tone_root(s : &str)->Option<(u8,&str)>{
    let mut it = s.chars();
    let first = it.next()?;
    let value : u8 = match first {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => {
            return None
        }
    };
    if let Some(second) = it.next(){
        match second{
            '#' => {
                Some((value + 1,&s[2..]))
            }
            'b' => {
                Some((value - 1,&s[2..]))
            }
            _=>{
                Some((value,&s[1..]))
            }
        }
    }else{
        Some((value,&s[1..]))
    }
}

//This one is ugly, maybe refactor
pub fn parse_chord<'i>(s : &'i str)->Option<(ChordEvent,&'i str)>{
    let mut s = s;
    let mut time = None;
    if let Some((ntime,ns)) = parse_time_offset(s){
        time = Some(ntime);
        s = ns;
    }
    let (root,mut s) = parse_tone_root(s)?;
    let mut kind = ChordKind::Major;
    let mut modifiers:Vec<ChordModifier> = Vec::new();
    let mut bass = None;
    if let Some('m') = s.chars().next(){
        kind = ChordKind::Minor;
        s = &s[1..];
    }

    loop{
        if let Some((modifier,ns)) = parse_keyword(s)  {
            s = ns;
            modifiers.push(modifier)
        }else {
            match s.chars().next() {
                Some('(')|Some(')')=>{
                    //Todo: actually check parens?
                    s = &s[1..];
                }
                _=>{
                    break;
                }
            }
        }
    }
    match s.chars().next(){
        Some('\'') | Some('/') =>{
            if let Some((nbass,ns))=  parse_tone_root(&s[1..]){
                s = ns;
                bass = Some(nbass);
            }
        }
        _=>{}
    }
    Some(
        (ChordEvent{
            root,
            kind,
            modifiers,
            time,
            bass
        },s)
    )
}

pub fn parse_time_offset(s: &str)->Option<(TimeOffset,&str)>{
    let mut s = s;
    let cur_char =  s.chars().next()?;
    let mut neg:i8 = 1;
    if cur_char == '<' {
        return Some((TimeOffset{
            beat : -1,
            num : 1,
            den : 2
        },&s[1..]));
    }
    if cur_char == '-'{
        neg = -1;
        s = &s[1..];
    }
    let (beat,s) = parse_uint_until(s)?;
    let beat = ((beat % 128) as i8)*neg;

    if let Some('+') =  s.chars().next(){
        if let Some ((num,ns)) = parse_uint_until(&s[1..]){
            if let Some(',') =  ns.chars().next(){
                if let Some((den,s)) = parse_uint_until(&ns[1..]){
                    return Some((TimeOffset{
                        beat,
                        num : num as u8,
                        den : den as u8
                    },s))
                }
            }
        }
    }

    Some((TimeOffset{
        beat,
        num : 0,
        den : 1
    },s))
}
macro_rules! seek_cascade {

    ($s:expr, $key:expr => $value:expr ) => {
        {
            seek($s,$key).map(|ns|($value,ns))
        }
    };

    ($s:expr, $key:expr => $value:expr, $($k:expr => $v:expr),+ ) => {
        {
            if let Some(ns) = seek($s,$key){
                Some(($value,ns))
            }else{
                seek_cascade!($s,$($k => $v),+)
            }
        }
    };
}

pub fn parse_keyword<'i>(s : &'i str) -> Option<(ChordModifier,&'i str)>{
    use ChordModifier::*;
    use ChordKeyword::*;

    //This is understandable and simple, but expensive
    if let Some(u) = seek_cascade!(s,
        "add11" => Keyword(Add11),
        "add2" => Keyword(Add2),
        "add4" => Keyword(Add4),
        "add9" => Keyword(Add9),
        "sus2" => Keyword(Sus2),
        "sus4" => Keyword(Sus4),
        "Maj" => Keyword(Maj),
        "6/9" => Keyword(K69),
        "aug" => Keyword(Aug),
        "dim" => Keyword(Dim),
        "11" => Keyword(K11),
        "13" => Keyword(K13),
        "9" => Keyword(K9), //woof
        "7" => Keyword(K7),
        "6" => Keyword(K6),
        "5" => Keyword(K5)
    ){
        return Some(u)
    }

    if let Some((kind,ns)) = seek_cascade!(s,
        "no" => ChordAlterationKind::No,
        "b" => ChordAlterationKind::Flat,
        "#" => ChordAlterationKind::Sharp
    ){
        let may_digits = 
            ns.get(..2)
            .and_then(
                |num_str|num_str.parse::<u8>().ok()
            ).or_else(||
                ns.get(..1)
                .and_then(
                    |num_str|num_str.parse::<u8>().ok()
                )
            );

        if let Some(degree) = may_digits{
            let delta = degree/10 + 1;
            return Some((Alteration(
                ChordAlteration{
                    kind,
                    degree
                }
            ),&ns[delta as usize..]))
        }
    }
    None
}


fn seek<'i>(s : &'i str,pattern : &str)->Option<&'i str>{
    let subs = s.get(..pattern.len())?;
    if subs != pattern{
        None
    }else{
        Some(&s[pattern.len()..])
    }
}

pub fn parse_uint_until(s : &str)->Option<(u32,&str)>{
    if !s.chars().next()?.is_ascii_digit(){
        return None;
    }
    let (i,val) = 
        s.char_indices()
        .take_while(|(_,c)|c.is_ascii_digit())
        .fold((0,0), |(idx,val),(i,c)|{
            (i,val*10+(c as u32 - '0' as u32))
        });
    Some((val,&s[i+1..]))
}