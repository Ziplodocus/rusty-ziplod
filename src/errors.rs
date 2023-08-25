
#[derive(Debug)]
pub enum Error {
    Plain(&'static str),
    Serenity(serenity::Error),
    Json(serde_json::Error),
    Cloud(cloud_storage::Error)
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