use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error occurred while saving")]
    Saving,

    #[error("error occurred while loading")]
    Loading,

    #[error("io error: {0}")]
    IO(std::io::Error),
}

impl Error {
    pub fn saving(_err: impl std::error::Error) -> Self {
        Self::Saving
    }

    pub fn loading(_err: impl std::error::Error) -> Self {
        Self::Loading
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}
