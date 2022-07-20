use shibim_base::SHBParseError;
use thiserror::Error;
use std::path::PathBuf;

#[derive(Error,Debug)]
#[error("Error loading file '{file}'")]
pub struct LoadError{
    pub file : String,
    #[source]
    pub detail : LoadErrorPayload
}

#[derive(Error,Debug)]
#[error("Post-processing error, {msg}")]
pub struct VisitorError{
    pub msg : String,
    detail : Option<Box<dyn std::error::Error>>
}

#[derive(Error,Debug)]
pub enum LoadErrorPayload{
    #[error(transparent)]
    IOError(#[from]std::io::Error),
    #[error("Synax error(s) '{0:?}'")] //Todo, this is duct tape
    ParseError(Vec<SHBParseError>)
}

