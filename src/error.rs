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
    GitReturnedNonZero(i32),
    GitTerminated,
    HomeDirectoryNotDetected,
    Io(std::io::Error),
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
            ErrorKind::GitReturnedNonZero(status) => {
                write!(f, "Git returned non-zero status code '{status}'")
            }
            ErrorKind::GitTerminated => write!(f, "Git terminated by signal"),
            ErrorKind::HomeDirectoryNotDetected => write!(f, "Home directory not detected"),
            ErrorKind::Io(e) => e.fmt(f),
            ErrorKind::InvalidArg(arg) => {
                write!(f, "unknown / invalid commandline argument")?;
                if let Some(arg) = arg {
                    write!(f, " '{arg}'")?;
                }
                Ok(())
            }
            ErrorKind::MissingRequiredArg(arg) => write!(f, "missing required argument '{arg}'"),
        }
    }
}

impl From<std::io::Error> for Box<Error> {
    fn from(e: std::io::Error) -> Box<Error> {
        Error::new(ErrorKind::Io(e))
    }
}
