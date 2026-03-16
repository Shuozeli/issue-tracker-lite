use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("group not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("group already exists: {0}")]
    AlreadyExists(String),

    #[error("failed precondition: {0}")]
    FailedPrecondition(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("internal error: {0}")]
    Internal(String),
}
