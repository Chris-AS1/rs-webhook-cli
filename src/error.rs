use thiserror;
use anyhow;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("Can't use those arguments together")]
    InvalidArgsError,
}
