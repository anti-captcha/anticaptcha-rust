//! Everything that can go wrong while solving a captcha.

/// Errors returned by this library.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A task is missing a required parameter, or a value is out of range.
    /// Nothing was sent to the API.
    #[error("{0}")]
    InvalidTask(String),

    /// The API could not be reached.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// The API answered with something that is not the JSON we expect.
    #[error("could not parse the API response: {message}; raw response: {raw}")]
    BadResponse {
        message: String,
        /// First 500 characters of what the API actually sent.
        raw: String,
    },

    /// The API answered with a non-zero `errorId`.
    /// See <https://anti-captcha.com/apidoc/errors> for the list of codes.
    #[error("API error {error_id} {error_code}: {description}")]
    Api {
        error_id: i64,
        error_code: String,
        description: String,
    },

    /// The task was still unsolved when the waiting limit ran out.
    #[error("the task was not solved in {0} seconds")]
    Timeout(u64),

    /// A captcha image file could not be read.
    #[error("could not read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

impl Error {
    /// The API error code, e.g. `ERROR_ZERO_BALANCE`, when this is an [`Error::Api`].
    pub fn api_error_code(&self) -> Option<&str> {
        match self {
            Error::Api { error_code, .. } => Some(error_code),
            _ => None,
        }
    }

    /// True when retrying the very same task could still succeed.
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Network(_) | Error::Timeout(_) => true,
            Error::Api { error_code, .. } => matches!(
                error_code.as_str(),
                "ERROR_NO_SLOT_AVAILABLE"
                    | "ERROR_SERVICE_OVERLOAD"
                    | "ERROR_RATE_LIMIT"
                    | "ERROR_CAPTCHA_UNSOLVABLE"
            ),
            _ => false,
        }
    }
}

/// Shorthand for a result carrying our [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
