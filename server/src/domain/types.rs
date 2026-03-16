use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("failed precondition: {0}")]
    FailedPrecondition(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl From<quiver_driver_core::RowError> for DomainError {
    fn from(err: quiver_driver_core::RowError) -> Self {
        DomainError::Internal(err.to_string())
    }
}

impl From<quiver_error::QuiverError> for DomainError {
    fn from(err: quiver_error::QuiverError) -> Self {
        DomainError::Internal(err.to_string())
    }
}

impl From<DomainError> for tonic::Status {
    fn from(err: DomainError) -> Self {
        match &err {
            DomainError::NotFound(_) => tonic::Status::not_found(err.to_string()),
            DomainError::InvalidArgument(_) => {
                tonic::Status::invalid_argument(err.to_string())
            }
            DomainError::AlreadyExists(_) => {
                tonic::Status::already_exists(err.to_string())
            }
            DomainError::FailedPrecondition(_) => {
                tonic::Status::failed_precondition(err.to_string())
            }
            DomainError::PermissionDenied(_) => {
                tonic::Status::permission_denied(err.to_string())
            }
            DomainError::Internal(_) => tonic::Status::internal(err.to_string()),
        }
    }
}
