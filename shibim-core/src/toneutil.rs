use shibim_base::*;
pub fn is_altered(a : NoteHeight) -> bool{
    let a  = a % 12;
    if a < 5 {
        a & 1 == 1
    }else{
        a & 1 == 0
    }

}

pub fn get_default_use_flat(a : NoteHeight)->bool{
    matches!(a, 3|5|8|10)
}
