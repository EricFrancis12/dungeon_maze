use std::num::{ParseFloatError, ParseIntError};
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    IO(std::io::Error),
    #[error("parsing error: {0}")]
    Parsing(String),
    #[error("error occurred while saving")]
    Saving,
    #[error("error occurred while loading")]
    Loading,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Self::Parsing(value.to_string())
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self::Parsing(value.to_string())
    }
}

macro_rules! error_impl {
    ($name:ident, $err_enum:ident) => {
        impl Error {
            pub fn $name(err: impl std::error::Error) -> Self {
                println!("{}", err);
                Self::$err_enum
            }
        }
    };
}

error_impl!(saving, Saving);
error_impl!(loading, Loading);
