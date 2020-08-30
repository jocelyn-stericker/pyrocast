use serde_xml_rs::Error as XmlError;
use std::error::Error;
use std::fmt;
use std::io::Error as IoError;
use std::sync::Arc;
use surf::http_types::Error as SurfError;
use surf::url::ParseError;

#[derive(Debug, Clone)]
/// A reason why content is unavailable.
pub enum StateError {
    Loading,
    DbError,
    UrlParseError(ParseError),
    NetError(Arc<SurfError>),
    IoError(Arc<IoError>),
    XmlError(Arc<XmlError>),
}

impl Default for StateError {
    fn default() -> Self {
        StateError::Loading
    }
}

impl PartialEq for StateError {
    fn eq(&self, other: &StateError) -> bool {
        matches!(
            (self, other),
            (StateError::Loading, StateError::Loading) |
                (StateError::DbError, StateError::DbError) |
                (StateError::UrlParseError(_), StateError::UrlParseError(_)) |
                (StateError::NetError(_), StateError::NetError(_)) |
                (StateError::IoError(_), StateError::IoError(_)) |
                (StateError::XmlError(_), StateError::XmlError(_))
        )
    }
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::Loading => write!(f, "Loading..."),
            StateError::DbError => write!(f, "Database error..."),
            StateError::UrlParseError(parse_error) => {
                write!(f, "Could not parse URL: {}", parse_error)
            }
            StateError::NetError(status_code) => write!(f, "Network error: {}", status_code),
            StateError::IoError(io_error) => write!(f, "IO Error: {}", io_error),
            StateError::XmlError(xml_error) => write!(f, "Xml Error: {}", xml_error),
        }
    }
}

impl Error for StateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            StateError::Loading => None,
            StateError::DbError => None,
            StateError::UrlParseError(parse_error) => Some(parse_error),
            StateError::NetError(surf_error) => Some(SurfError::as_ref(surf_error)),
            StateError::IoError(io_error) => Some(io_error.as_ref()),
            StateError::XmlError(xml_error) => Some(xml_error.as_ref()),
        }
    }
}

impl From<ParseError> for StateError {
    fn from(parse_error: ParseError) -> Self {
        StateError::UrlParseError(parse_error)
    }
}

impl From<SurfError> for StateError {
    fn from(parse_error: SurfError) -> Self {
        StateError::NetError(Arc::new(parse_error))
    }
}

impl From<IoError> for StateError {
    fn from(io_error: IoError) -> Self {
        StateError::IoError(Arc::new(io_error))
    }
}

impl From<XmlError> for StateError {
    fn from(xml_error: XmlError) -> Self {
        StateError::XmlError(Arc::new(xml_error))
    }
}
