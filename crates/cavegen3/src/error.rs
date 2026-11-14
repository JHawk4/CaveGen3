use std::fmt;

/// An error loading or parsing game data, or understanding a sublevel name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(pub String);

impl Error {
    pub(crate) fn new(msg: impl Into<String>) -> Self {
        Error(msg.into())
    }

    /// Adds context in front of the message, e.g. the file being parsed.
    pub(crate) fn context(self, ctx: impl fmt::Display) -> Self {
        Error(format!("{ctx}: {}", self.0))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
