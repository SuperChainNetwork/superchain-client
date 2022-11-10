/// A Result alias, that uses [`Error`] as the default error
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A collections of errors that can occur when using this crate
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The server sent and unexpected WebSocket message
    /// This should usually not happen
    #[error("The server sent an unexpected message")]
    UnexpectedMessage,
    /// The server send a malformed message
    /// This should usually not happen
    #[error("The server sent a malformed message")]
    UnexpectedMessageFormat,
    /// The server sent a response for a requests without a listener
    /// This should usually not happen
    #[error("The server sent a response for a non existing request")]
    UnknownResponseId,
    /// The maximum limit of 256 concurrent requests was reached
    ///
    /// Note, that requests with open end (live streams) can currently not be unsubscribed.
    /// If you run into that you could create a new WebSocket connection to clean up
    #[error("The maximum limit of 256 concurrent requests was reached")]
    MaxConcurrentRequestLimitReached,
    /// The backend websocket service shutdown
    /// This happens, when the server closes the connection
    #[error("The backend service shut down")]
    BackendShutDown,
    /// The server sent an error message as part of the response
    #[error("An error occurred while processing the request: {0}")]
    ErrorMsg(String),
    /// The websocket connection was closed by the server
    #[error("The websocket connection was closed")]
    ConnectionClosed,

    /// An error encountered during csv parsing
    #[error(transparent)]
    CsvAsync(#[from] csv_async::Error),
    /// An IO error
    #[error(transparent)]
    IO(#[from] std::io::Error),
    /// An error encountered during making HTTP requests
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    /// An error encountered during cbor parsing
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    /// An error encountered during websocket handling
    #[error(transparent)]
    Tungstenite(#[from] tungstenite::Error),
    /// An error encountered during url parsing
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error("an unexpected error occurred: {0}")]
    Custom(String),
}
