use diesel::result::Error as DieselError;
use discord::Error as DiscordError;
use hyper::Error as HyperError;
use serde_json::Error as JsonError;
use std::num::ParseIntError;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    Decode,
    Diesel,
    Discord(DiscordError),
    Hyper(HyperError),
    Json(JsonError),
    YoutubeDL(String),
}

impl From<DieselError> for Error {
    fn from(_e: DieselError) -> Error {
        Error::Diesel
    }
}

impl From<HyperError> for Error {
    fn from(e: HyperError) -> Error {
        Error::Hyper(e)
    }
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Error {
        Error::Json(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(_e: ParseIntError) -> Error {
        Error::Decode
    }
}
