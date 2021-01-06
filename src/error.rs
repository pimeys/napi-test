use deadpool::managed::PoolError;
use napi::Status;
use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    Tiberius(tiberius::error::Error),
    Timeout,
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tiberius(e) => write!(f, "{}", e),
            Self::Io(e) => write!(f, "{}", e),
            Self::Timeout => write!(f, "Timeout"),
        }
    }
}

impl From<tiberius::error::Error> for Error {
    fn from(err: tiberius::error::Error) -> Self {
        Self::Tiberius(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<PoolError<Error>> for Error {
    fn from(e: PoolError<Error>) -> Self {
        match e {
            PoolError::Timeout(_) => Error::Timeout,
            PoolError::Backend(e) => e,
        }
    }
}

impl From<Error> for napi::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::Tiberius(e) => napi::Error::new(Status::GenericFailure, e.to_string()),
            Error::Io(e) => napi::Error::new(Status::GenericFailure, e.to_string()),
            Error::Timeout => napi::Error::new(Status::GenericFailure, String::from("Timeout")),
        }
    }
}

impl std::error::Error for Error {}
