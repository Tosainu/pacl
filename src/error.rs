use std::fmt;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Box<Error> {
        Box::new(Error { kind })
    }
}

pub type Result<T> = std::result::Result<T, Box<Error>>;

#[derive(Debug)]
pub enum ErrorKind {
    InvalidUrl(url::ParseError),
    InvalidArg(Option<String>),
    MissingRequiredArg(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::InvalidUrl(e) => e.fmt(f),
            ErrorKind::InvalidArg(arg) => {
                write!(f, "unknown / invalid commandline argument")?;
                if let Some(arg) = arg {
                    write!(f, " '{}'", arg)?;
                }
                Ok(())
            }
            ErrorKind::MissingRequiredArg(arg) => write!(f, "missing required argument '{}'", arg),
        }
    }
}

impl From<url::ParseError> for Box<Error> {
    fn from(e: url::ParseError) -> Box<Error> {
        Error::new(ErrorKind::InvalidUrl(e))
    }
}

