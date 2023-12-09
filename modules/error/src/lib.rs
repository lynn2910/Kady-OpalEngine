use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

/// Will be used each time an error can occur
pub type Result<T> = core::result::Result<T, Error>;

/// Represent an error
#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    Gateway(GatewayError),
    Api(ApiError),
    Event(EventError),
    Config(ConfigError),
    Fs(FileError),
    Archive(ArchiveError),
    Database(DatabaseError),
    Model(ModelError),
    Runtime(RuntimeError)
}

impl From<serde_json::error::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Api(ApiError::InvalidJson(value.to_string()))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

/// Represent an error that can occur while the runtime is active
#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeError {
    pub context: Option<String>,
    pub target: Option<String>,
    pub reason: String,
    pub trace: Option<Vec<String>>,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for RuntimeError {}


impl RuntimeError {
    pub fn new(reason: impl ToString) -> Self {
        Self {
            reason: reason.to_string(),
            trace: None,
            target: None,
            context: None
        }
    }

    pub fn with_target(mut self, target: impl ToString) -> Self {
        self.target = Some(target.to_string());
        self
    }

    pub fn with_context(mut self, context: impl ToString) -> Self {
        self.context = Some(context.to_string());
        self
    }

    pub fn push_trace(mut self, trace: impl ToString) -> Self {
        if self.trace.is_none() { self.trace = Some(Vec::new()) };
        self.trace.as_mut().unwrap().push(trace.to_string());
        self
    }
}

/// Represent an error that can occur inside a model
#[derive(Debug, Serialize, Deserialize)]
pub enum ModelError {
    InvalidSnowflake(String),
    MissingField(String),
    InvalidPayload(String),
    InvalidTimestamp(String),
}

/// Represent an error that can occur inside the config system
#[derive(Debug, Serialize, Deserialize)]
pub enum ConfigError {
    InvalidFile(String),
    CannotReadFile(String),
    InvalidConfig(String),
    CannotWriteFile(String),
}

/// Represent an error that can occur inside the event system
#[derive(Debug, Serialize, Deserialize)]
pub enum EventError {
    TooManyListeners(String),
    MissingField(String),
    Runtime(String),
}

/// Represent an error that can occur inside the gateway system
#[derive(Debug, Serialize, Deserialize)]
pub enum GatewayError {
    ShardConnectionError(String),
    ShardMessageError(String),
    ParsingError(String),
    PayloadError(String),
    /// Error received when we can't send/receive messages through the UnboundedChannel
    InternChannelError(String),
    ShardNotFound(String),
}

/// Represent an error that can occur inside the api
#[derive(Debug, Serialize, Deserialize)]
pub enum ApiError {
    RequestError(String),
    RequestStatus(String),
    InvalidJson(String),
    NoResponse(String),
    InvalidResource(String),
    MutexLock(String),
    ChannelRecv,
    ChannelClosed,
    ChannelSend(String),
    TooManyRetry,
    ConversionError(String),
    Deserialize(String),
}

/// Represent an error that can occur inside the archive system
#[derive(Debug, Serialize, Deserialize)]
pub enum ArchiveError {
    CorruptedArchive(String),
    UnsupportedFormat(String),
    NoArchive(String),
    InvalidUtf8Sequence(String),
    InvalidHeader(String),
    InvalidBody(String),
    CannotSerializeBody(String),
    InvalidBodyValue(String),
}

/// Represent an error that can occur with the file system
#[derive(Debug, Serialize, Deserialize)]
pub enum FileError {
    CannotReadFile(String),
    CannotWriteFile(String),
    InvalidFile(String),
    InvalidUtf8Sequence(String),
    NoFile(String),
    IOError(String),
    InvalidPath(String),
    CannotReadDir(String),
}


#[derive(Debug, Serialize, Deserialize)]
pub enum DatabaseError {
    /// Returned when the connexion informations weren't found
    InvalidCredentials(String),
    /// When the connection cannot be established
    CannotConnect(String),
    /// Returned when a query has failed
    QueryError(String),
    /// Returned when the connection cannot be acquired from the pool
    CannotAcquireConnection(String),
    CannotParseDynamicRequestTable(String),
}