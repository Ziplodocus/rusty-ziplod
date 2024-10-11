use serenity::framework::standard::CommandError;

#[derive(Debug)]
pub enum Error {
    Plain(&'static str),
    Serenity(serenity::Error),
    Json(serde_json::Error),
    Cloud(cloud_storage::Error),
    Io(std::io::Error),
    Reqwest(reqwest::Error),
    Songbird(songbird::error::JoinError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Plain(str) => write!(f, "{}", str),
            Error::Serenity(err) => write!(f, "{:?}", err),
            Error::Json(err) => write!(f, "{:?}", err),
            Error::Cloud(err) => write!(f, "{:?}", err),
            Error::Io(err) => write!(f, "{:?}", err),
            Error::Reqwest(err) => write!(f, "{:?}", err),
            Error::Songbird(err) => write!(f, "{:?}", err),
        }
    }
}

impl From<serenity::Error> for Error {
    fn from(value: serenity::Error) -> Self {
        Error::Serenity(value)
    }
}
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Json(value)
    }
}
impl From<cloud_storage::Error> for Error {
    fn from(value: cloud_storage::Error) -> Self {
        Error::Cloud(value)
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Reqwest(value)
    }
}

impl From<songbird::error::JoinError> for Error {
    fn from(value: songbird::error::JoinError) -> Self {
        Error::Songbird(value)
    }
}

impl From<Error> for CommandError {
    fn from(value: Error) -> Self {
        CommandError::from(value.to_string())
    }
}
