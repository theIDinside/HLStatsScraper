use reqwest::Error;
use std::fmt::Formatter;

/// These are type aliases meant for documentation of use for BuilderError
pub type Message = &'static str;
pub type HTMLNodeText = String;
pub type FieldName = String;
#[allow(dead_code)]
pub enum BuilderError {
    REQWEST(reqwest::Error),
    Parse(usize, HTMLNodeText, Message),
    ValueIncorrect(usize, FieldName, Message),
    GameIncomplete(usize, Vec<String>),
    StringOperationFailed(String, Message),
    CouldNotParseGameID(String),
    WrongURLStringFormat(String),
    GamePostponed,
    ChannelError
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderError::REQWEST(req_err) => {
                write!(f, "{}", req_err)
            },
            BuilderError::Parse(game_id, node_text, msg) => {
                write!(f, "Game ID: {}. Node text parsing failed on: {}. Value parsing for: {}", game_id, node_text, msg)
            },
            BuilderError::ValueIncorrect(game_id, field, msg) => {
                write!(f, "Game ID: {}. Value incorrect for {}. {}", game_id, field, msg)
            },
            BuilderError::GameIncomplete(game_id, fields) => {
                write!(f, "Game ID: {} incomplete. Missing fields: {:?}", game_id, fields)
            },
            BuilderError::StringOperationFailed(datum, msg) => {
                write!(f, "String operation failed on '{}'. Operation: {}", datum, msg)
            },
            BuilderError::CouldNotParseGameID(parse_string) => {
                write!(f, "Game ID could not be parsed. Data parsed: '{}'", parse_string)
            },
            BuilderError::WrongURLStringFormat(url_string) => {
                write!(f, "URL string format was faulty: {}", url_string)
            },
            BuilderError::GamePostponed => {
                write!(f, "Game was postponed")
            }
            BuilderError::ChannelError => {
                write!(f, "Channel error")
            }
        }
    }
}

impl From<reqwest::Error> for BuilderError {
    fn from(e: Error) -> Self {
        BuilderError::REQWEST(e)
    }
}