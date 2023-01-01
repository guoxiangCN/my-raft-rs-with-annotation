use std::io;
use thiserror::Error;

#[derive(Error,Debug)]
pub enum Error {

    #[error("io {0}")]
    Io(#[from] io::Error),

    #[error("store error")]
    Store,
    
    #[error("step local msg error")]
    StepLocalMsg,
    
    #[error("step peer not found error")]
    StepPeerNotFound,
    
    #[error("proposal dropped error")]
    ProposalDropped,
    
    #[error("config invalid")]
    ConfigInvalid,
    
    #[error("codec error")]
    Codec,
}

pub type Result<T> = std::result::Result<T, Error>;
