use tonic::Status;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum CpassError {
    /// If the request was invalid or malformed.
    #[error("the request was invalid {0}")]
    InvalidRequest(String),

    /// If the username and password combination did not match when attempting to authenticate.
    #[error("invalid username or password")]
    InvalidUsernameOrPassword,

    /// If a registration was attemted, but the email address already exists in the database.
    #[error("a user with the email {0} already exists")]
    UserAlreadyExists(String),

    /// An error occured when validating or generating a JWT.
    #[error("invalid token")]
    InvalidToken(#[from] jsonwebtoken::errors::Error),

    /// An error occured when connection to or using the database.
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),

    /// An error occured with the Argon2id hashing implementation.
    #[error("hashing error")]
    HashingError(#[from] argon2::Error),

    /// Any other, unknown error sources.
    #[error("{0}")]
    Unknown(#[source] Box<dyn std::error::Error>),
}

impl From<CpassError> for tonic::Status {
    fn from(cpass_error: CpassError) -> Self {
        let error = format!("{:?}", cpass_error);
        match cpass_error {
            CpassError::InvalidRequest(_) => Status::invalid_argument(error),
            CpassError::InvalidUsernameOrPassword => Status::unauthenticated(error),
            CpassError::UserAlreadyExists(_) => Status::invalid_argument(error),
            CpassError::InvalidToken(_) => Status::unauthenticated(error),
            CpassError::DatabaseError(_) => Status::unavailable(error),
            CpassError::HashingError(_) => Status::unauthenticated(error),
            CpassError::Unknown(_) => Status::unknown(error),
        }
    }
}
