use std::error::Error;
use std::fmt;

use ring::error::Unspecified;

#[derive(Debug)]
pub enum KeyError {
    Failure(String),
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KeyError::Failure(reason) => write!(f, "Failure: {:?}", reason),
        }
    }
}

impl Error for KeyError {}

impl From<Unspecified> for KeyError {
    fn from(_unspecified: Unspecified) -> Self {
        KeyError::Failure(String::from("Unspecified ring error"))
    }
}
