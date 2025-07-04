use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Parse(String),
    SteamNotFound,
    SteamConfigNotFound(PathBuf),
    InvalidAppId(String),
    InvalidManifest(String),
    LibraryNotFound(PathBuf),
    FileSystemError(String),
    PermissionDenied(PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "I/O error: {}", err),
            Error::Parse(msg) => write!(f, "Parse error: {}", msg),
            Error::SteamNotFound => write!(
                f,
                "Steam installation not found. Please ensure Steam is installed and running."
            ),
            Error::SteamConfigNotFound(path) => {
                write!(f, "Steam configuration not found at: {}", path.display())
            }
            Error::InvalidAppId(id) => {
                write!(f, "Invalid AppID: {}. AppIDs must be positive numbers.", id)
            }
            Error::InvalidManifest(msg) => write!(f, "Invalid manifest file: {}", msg),
            Error::LibraryNotFound(path) => {
                write!(f, "Steam library not found at: {}", path.display())
            }
            Error::FileSystemError(msg) => write!(f, "File system error: {}", msg),
            Error::PermissionDenied(path) => {
                write!(f, "Permission denied accessing: {}", path.display())
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::PermissionDenied => Error::PermissionDenied(PathBuf::new()),
            _ => Error::Io(err),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
