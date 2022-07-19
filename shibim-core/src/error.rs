use shibim_base::SHBParseError;
use thiserror::Error;

#[derive(Error,Debug)]
#[error("Error loading file '{file}'")]
pub struct LoadError{
    pub file : String,
    #[source]
    pub detail : LoadErrorPayload
}
#[derive(Error,Debug)]
pub enum LoadErrorPayload{
    #[error(transparent)]
    IOError(#[from]std::io::Error),
    #[error("Synax error '{0:?}'")] //Todo, this is duct tape
    ParseError(Vec<SHBParseError>)
}