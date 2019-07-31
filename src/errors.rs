use std::fmt;
use std::io;

use failure::{Backtrace, Context, Fail};

/// A type alias for handling errors throughout the kvs library.
pub type Result<T> = std::result::Result<T, Error>;

/// An error that can occur while interacting with the kvs.
#[derive(Debug)]
pub struct Error {
    ctx: Context<ErrorKind>,
}

impl Error {
    /// Return the kind of this error.
    pub fn kind(&self) -> &ErrorKind {
        self.ctx.get_context()
    }

    /// Shortcut for constructing an Io error.
    pub(crate) fn io(err: io::Error) -> Error {
        Error::from(ErrorKind::Io(err.to_string()))
    }

    /// Shortcut for constructing a Serde error.
    pub(crate) fn serde(err: serde_json::Error) -> Error {
        Error::from(ErrorKind::Serde(err.to_string()))
    }

    /// Shortcut for constructing a Serde error from a Bincode error.
    pub(crate) fn bincode(err: bincode::Error) -> Error {
        Error::from(ErrorKind::Serde(err.to_string()))
    }

    /// Shortcut for constructing a CorruptDatabase error
    pub(crate) fn corrupt_database(msg: String) -> Error {
        Error::from(ErrorKind::CorruptDatabase(msg))
    }

    // /// Shortcut for constructing a KeyDoesNotExist error.
    // pub(crate) fn key_does_not_exist<T: AsRef<str>>(key: T) -> Error {
    //     Error::from(ErrorKind::KeyDoesNotExist(key.as_ref().to_string()))
    // }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.ctx.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.ctx.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.ctx.fmt(f)
    }
}

/// The error type for the class
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// An unexpected I/O error occurred.
    Io(String),
    /// An error occured while serializing or deserializing data
    Serde(String),
    /* /// An error while looking for an entry in the key-value store.
     * ///
     * /// The key does not exist.
     * KeyDoesNotExist(String), */
    /// The database has been corrupted (has an inconsistent state).
    CorruptDatabase(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::Io(ref msg) => write!(f, "I/O error: {}", msg),
            ErrorKind::Serde(ref msg) => write!(f, "Serde error: {}", msg),
            ErrorKind::CorruptDatabase(ref msg) => {
                write!(f, "CorruptDatabase error: {}", msg)
            } /* ErrorKind::KeyDoesNotExist(ref key) => {
               *     write!(f, "key does not exist: {}", key)
               * } */
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error::from(Context::new(kind))
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(ctx: Context<ErrorKind>) -> Error {
        Error { ctx }
    }
}
