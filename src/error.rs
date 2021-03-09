use std::fmt::{Debug, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    Utf8Error(std::str::Utf8Error),
    ParserError(ErrorKind),
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self::ParserError(kind)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::IoError(ref err) => Display::fmt(&err, f),
            Self::Utf8Error(ref err) => Display::fmt(&err, f),
            Self::ParserError(ref err) => write!(f, "Parser error occured: {:?}", err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    CorruptOrInvalidFile(String),
    CorruptOrInvalidRecord(String),
    InvalidFlags,
    NomError,
    UnconsumedBytes(usize),
}

impl ErrorKind {
    fn as_str(&self) -> &str {
        match self {
            Self::CorruptOrInvalidFile(_) => "Corrupt or invalid plugin/master file",
            Self::CorruptOrInvalidRecord(_) => "Corrupt or invalid record",
            Self::InvalidFlags => "Cannot parse flags",
            Self::NomError => "Error in nom parser",
            Self::UnconsumedBytes(_) => "Parser left bytes unconsumed",
        }
    }
}
