use shibim_base::SHBParseError;
use thiserror::Error;


pub struct LoadError{
    pub file : String,

    pub detail : LoadErrorPayload
}
#[derive(Error,Debug)]
pub enum LoadErrorPayload{
    #[error(transparent)]
    IOError(#[from]std::io::Error),
    #[error("Error parsing shb file '{0:?}'")] //Todo, this is duct tape
    ParseError(Vec<SHBParseError>)
}