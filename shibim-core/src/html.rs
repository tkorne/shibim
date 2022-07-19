use markup;
use shibim_base as base;
use crate::i18n::*;
use crate::toneutil;
/* 
struct SongHTMLOptions{
    use_header : bool,
    remove_edit_buttons : bool,
    force_flats : bool
}*/

fn get_orders_serialized(song : & base::Song)->String{
    serde_json::to_string(
        &song.orders.keys()
        .collect::<Vec<&String>>()
    )
    .unwrap_or_default()
}

fn get_default_order(song : &base::Song)->Option<(&str, &Vec<usize>)>{
    song.orders.get_key_value("default")
    .or_else(
        || song.orders.get_key_value("full")
    ).or_else(
        || song.orders.iter().next()
    ).map(|(k,v)|(k.as_str(),v))
}
/*
fn create_default_order(song : &base::Song)->Vec<usize>{
   (0..song.sections.len()).collect()
}*/
markup::define!(
    Song<'i>(song: &'i base::Song){
        article [
                "data-tonic" = song.tonic,
                "data-orders" = get_orders_serialized(song),
                "data-mode" = {
                    if let base::TonicKind::Minor = song.tonic_kind{
                        "m"
                    }else{
                        ""
                    }
                }
            ]{
            {SongHeader{name : &song.name, tonic : song.tonic, mode : song.tonic_kind}}
        }
        @if let Some((_name,indexes)) = get_default_order(song){
            {SongOrder{
                song,
                order : indexes,
                use_flats: toneutil::get_default_use_flat(song.tonic)
            }}
        }

    }
    CompiledSong<'i>(song: base::SongRef<'i>){
        article [
                "data-tonic" = song.tonic,
                "data-mode" = {
                    if let base::TonicKind::Minor = song.tonic_kind{
                        "m"
                    }else{
                        ""
                    }
                }
            ]{
            {SongHeader{name : song.name, tonic : song.tonic, mode : song.tonic_kind}}
        }
        @for section in  song.sections{
            {Section{section,use_flats : toneutil::get_default_use_flat(song.tonic)}}
        }
    }
    SongHeader<'i>(name : &'i str, tonic : base::NoteHeight, mode : base::TonicKind){
        $ "u-song-title-box"{
            @if !name.is_empty(){
                $ "u-song-name"{
                    h2 {{name}}
                }
            }
            button . "tone-button" {
                {ChordRoot{root : *tonic, use_flats : true}}
                @if let base::TonicKind::Minor = mode{
                    "m"
                }
            }
            {UtilButtonsHTML{}}
            button . "edit-button"{

            }
        }
    }    
    SongOrder<'i,'b>(song : &'i base::Song,order : &'b Vec<usize>,use_flats : bool){
        @for may_section in order.iter().map(|x|song.sections.get(*x)){
            @if let Some(section) = may_section{
                {Section{section, use_flats : *use_flats} }
            }else{
                {eprintln!("Corrupted section information!");""} //Todo
            }
        }
    }
    Section<'i>(section : &'i base::Section, use_flats : bool){
        $ "section" . "u-section" ["data-id" = &section.name]{
            $ "u-section-title-box"{
                $ "u-title-background"{
                    $ "u-section-name"{
                        {&section.name}
                    }
                    h3{
                        {&section.description}  
                    }
                    @if section.name.starts_with('C') {
                        $"u-chorus_mark"{}
                    }
                }
                {UtilButtonsHTML{}}
            }
            @for subsection in &section.subsections{
                {Subsection{subsecion : subsection, use_flats : *use_flats}}
            }
        }
    }
    Subsection<'i>(subsecion : &'i base::Subsection, use_flats : bool){
        $ "u-s"{
            @if let Some(re) = subsecion.metadata.get("ref") {
                $"u-ref"{
                    {re}
                }
            }
            @for line in &subsecion.lines{
                {Line{line, use_flats : *use_flats}}
            }
        }
    }
    Line<'i>(line : &'i base::Line, use_flats : bool){
        @match line {
            base::Line::Lyrics(lyrics)=>{
                $"u-l" . lyr {
                @for measure in lyrics{
                    @match measure{
                        Some(measure)=>{
                            $"u-m"{
                            @for block in measure{
                                $"u-b"{
                                $"u-y"{
                               
                                    @for fragment in block{
                                        {LyricEvent{evt : fragment}}
                                    }
                                }
                                }
                            }
                            }
                        }
                        None =>{
                            $"u-m" . empty{
                            }
                        }
                    }
                }
                }
            }
            base::Line::Chords(chords)=>{
                @let mut on_parens = false;
                $"u-l" . chr {
                @for measure in chords{
                    @match measure{
                        Some(measure)=>{
                            $"u-m"{
                            @for block in measure{
                                $"u.b"{
                                    $"u-c"{
                                    @for fragment in block{
                                        @if let base::MusicEvent::OpenParen = fragment{
                                            {on_parens = true;""}
                                        }else if let base::MusicEvent::CloseParen = fragment{
                                            {on_parens = false;""}
                                        }
                                        {MusicEvent{evt : fragment, use_flats : *use_flats, is_par: on_parens}}
                                    }
                                    }
                                }
                            }
                            }
                        }
                        None =>{
                            $"u-m" . empty{

                            }
                        }
                    }
                }
                }
            }
            base::Line::Mixed(elems)=>{
                @let mut on_parens = false;
                $"u-v" {
                @for measure in elems{
                    @match measure{
                        Some(measure)=>{
                            $"u-m"{
                            @for (chord_block,lyric_block) in measure{
                                $"u-b"{
                                    $"u-c"{
                                        @for evt in chord_block{
                                            @if let base::MusicEvent::OpenParen = evt{
                                                {on_parens = true;""}
                                            }else if let base::MusicEvent::CloseParen = evt{
                                                {on_parens = false;""}
                                            }
                                            {MusicEvent{evt, use_flats : *use_flats,is_par : on_parens}}
                                        }
                                    }
                                    $"u-l"{
                                        @for evt in lyric_block{
                                            {LyricEvent{evt}}
                                        }
                                    }
                                }
                            }
                            }
                        }
                        None =>{
                            $"u-m" . empty{
                            }
                        }
                    }
                }
                }
            }
        }
    }

    TimeOffset<'i>(time : &'i base::TimeOffset){
        $ "u-tim"{
            @if time.beat == 0-1{
                @if time.den == 2 && time.num == 1{
                    {"<"}
                }else{
                    span {
                        "-"
                        sup {{time.num}}
                        sub {{time.den}}
                    }
                }
            }else{
                { time.beat }

                @if time.den == 0{

                }else if time.den == 2 && time.num == 1{
                    {"\""}
                }else{
                    span {
                        sup {{time.num}}
                        sub {{time.den}}
                    }
                }
            }
        }
    }
    
    ChordRoot(root : base::NoteHeight, use_flats : bool){
        @if toneutil::is_altered(*root){
            @if *use_flats{
                {base::FLAT_TONIC_NAMES[*root as usize]}
                $"u-a" {
                    "b"
                }
            }else{
                {base::SHARP_TONIC_NAMES[*root as usize]}
                
                $"u-a"{
                    "#"
                }
            }
        }else{
            {base::SHARP_TONIC_NAMES[*root as usize]}
        }
    }


    ChordEvent<'i>(chord : &'i base::ChordEvent, use_flats : bool){
    
        @if let Some(time) = &chord.time {

                {TimeOffset{time}}
        }

        $ "u-r" {
            {ChordRoot{root : chord.root,use_flats : *use_flats}}
        }

        @if let base::ChordKind::Minor = chord.kind{
            $ "u-n"{"m"}
        }
        @for modifier in &chord.modifier{
            {ChordModifier{modifier}}
        }
        @if let Some(bass) = chord.bass{
            $"u-bas" {
                    "/"
                    $ "u-r" {
                    {ChordRoot{root : bass, use_flats: *use_flats}}
                } 
            }
        }
    }

    ChordModifier<'i>(modifier : &'i base::ChordModifier){
        @match modifier{
            base::ChordModifier::Keyword(keyword) =>{
                @match keyword{
                    base::ChordKeyword::Maj |
                     base::ChordKeyword::K5 |
                     base::ChordKeyword::K6 |
                     base::ChordKeyword::K7 |
                     base::ChordKeyword::K9 | 
                     base::ChordKeyword::K13 => {
                        //Short for keyword
                        $"u-k" {{CHORD_KEYWORDS_NAMES[keyword]}}
                    }
                    _=>{
                        //Short for keyword-long
                        $"u-kl" {{CHORD_KEYWORDS_NAMES[keyword]}}
                    }
                }
            }
            base::ChordModifier::Alteration(alter)=>{
                $ "u-alt"  {
                    @match alter.kind {
                        base::ChordAlterationKind::Flat =>{"b"}
                        base::ChordAlterationKind::Sharp => {"#"}
                        base::ChordAlterationKind::No => {"no"}
                    }
                    {alter.degree}                   
                }
                
            }
        }
    }
    LyricEvent<'i>(evt: &'i base::LyricEvent){
        @match evt{
            base::LyricEvent::LyricText(text)=>{
                {text}
            }
            base::LyricEvent::LyricBreak=>{
                $"u-lbrk"{
                    "\u{200B}"
                }
            }
        }
    }
    MusicEvent<'i>(evt : &'i base::MusicEvent, use_flats : bool, is_par : bool){
        @match evt{
            base::MusicEvent::ChordEvent(evt)=>{
                @if *is_par{
                    $ "u-h"{
                        {ChordEvent{chord : evt, use_flats : *use_flats}}
                    }
                }else{
                    $ "u-h" . "par"{
                        {ChordEvent{chord : evt, use_flats : *use_flats}}
                    }
                }
            }
            base::MusicEvent::MelodyEvent(evt)=>{
                $ "u-mel"{
                    {MelodyEvent{melody:evt, use_flats : *use_flats}}
                }
            }
            base::MusicEvent::RepeatMeasure=>{
                $ "u-sym"{
                    {"·"} //TODO: ooo
                }
            }
            base::MusicEvent::Annotation(text)=>{
                $ "u-ann"{
                    {text}
                }
            }
            base::MusicEvent::NumberedMeasure(num)=>{
                $ "u-num"{
                    {num}
                }
            }
            _=>{
                {eprintln!("Not implemented!");""} //TODO ooo
            }
        }
    }


    MelodyEvent<'i>(melody : &'i Vec<base::NoteHeight>, use_flats : bool){

        @for (i,note) in melody.iter().enumerate(){
            $"u-r"{
                {ChordRoot{root:*note,use_flats : *use_flats}} 
            }
            {if i < melody.len() - 1 {" "} else {""} }
        }
    }

    UtilButtonsHTML(){
        button . "collapse-button" {

        }
        button ."moveup-button"{

        }
        button . "movedown-button"{

        }
        button . "remove-button" [style="display:none;"]{

        }
    }

    HeadHTML(){
        head{
            meta [charset ="utf-8"];
            meta [name="viewport", content="width=device-width"];
            link [rel="stylesheet", href="../css/style.css"];
        }
    }

    HeadIdxHTML(){
        head{
            meta [charset ="utf-8"];
            meta [name="viewport", content="width=device-width"];
            link [rel="stylesheet", href="css/index.css"];
        }
    }

    ConfButtons(){
        div . "conf-bar" {
            a . "conf-btn" # "index-btn" [href="../songs.html"] {"Índice"}
            button ."conf-btn" # "save-btn"{"Descargar"}
            button ."conf-btn" # "hidecontrol-btn" {"Edición"}
            button ."conf-btn" # "col-btn"{"Columnas"}
            button ."conf-btn" # "hidechord-btn" {"Acordes"}
            button ."conf-btn" # "hidelyrics-btn" {"Letra"}
            button ."conf-btn" # "small-btn" {"Más pequeño"}
            button ."conf-btn" # "big-btn" {"Más grande"}
            button ."conf-btn" # "margin-btn" {"Margen"}
            button ."conf-btn" # "dark-btn" {"Color"}
            a . "conf-btn" # "pres-btn" [href="../present.html",target="_blank"] {"Presentacion"}
            button ."conf-btn" # "conn-btn" {"Conectar"}
        }
    }

);

//Do not want to add dependency for this non-essential
//one time 
/*
mod date{
    fn is_leap(year : i64) -> bool{
        return year % 4 == 0 &&
               (year % 100 != 0 ||
               year % 400 == 0)
               
    }

    fn leap_count_1600(year : i64) -> i64{
        let delta = year - 1600;
        let leap_4 = (delta + 3)/4;
        let common_100 = (delta + 99)/100;
        let leap_400 = (delta + 399)/400;
        return leap_4 - common_100 + leap_400;
    }
    const ACC_DAYS_MONTH : [i64 ; 12] = 
      [ 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    //TODO: sad is to add spaces to string constants
    const DAY_NAMES: &[&str] = &["Dom ","Lun ","Mar ","Mie ","Jue ","Vie ","Sab "];

    fn day_count_1600_upto(year : i64) -> i64{
        return (year - 1600)*365 + leap_count_1600(year);
    }

    fn parse_date(datestr : &str) -> Option<(i64,u8,u8)>{
        if datestr.len() < 10 {
            return None;
        }
        let year : i64  =(datestr.get(0..4)?).parse().ok()?;
        let month : u8  = (datestr.get(5..7)?).parse().ok()?;
        let day : u8 = (datestr.get(8..10)?).parse().ok()?;
        
        return Some((year,month,day));
    }
    fn week_day(year : i64, month : u8, day : u8) -> Option<u8>{
        if year < 1600 || month > 12 || day > 31 || month < 1 || day < 1 {
            return None;
        }
        let day_ref = 6;
        let mut days_from_ref = day_count_1600_upto(year);
        days_from_ref += ACC_DAYS_MONTH[(month-1) as usize];
        if is_leap(year) && month > 2 {
            days_from_ref += 1;
        }
        days_from_ref = days_from_ref + (day as i64) - 1;
        return Some( ((days_from_ref + day_ref) % 7) as u8);
    }

    pub fn try_week_day_str(datestr: &str) -> &str{
        let date = parse_date(datestr).and_then(|(y,m,d)|{
            week_day(y,m,d)
        });
        match date{
            None => "",
            Some(n) => DAY_NAMES[n as usize]
        }
    }

}
*/