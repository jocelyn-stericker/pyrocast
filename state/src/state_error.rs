use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A reason why content is unavailable.
pub enum StateError {
    Loading,
    DbError,
    NetError,
}

impl Default for StateError {
    fn default() -> Self {
        StateError::Loading
    }
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::Loading => write!(f, "Loading..."),
            StateError::DbError => write!(f, "Database error..."),
            StateError::NetError => write!(f, "Network error..."),
        }
    }
}

impl Error for StateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
