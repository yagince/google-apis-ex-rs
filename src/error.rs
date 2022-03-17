use std::env;
use std::io;

use reqwest::header::ToStrError;

use crate::drive::client::GoogleDriveError;
use crate::storage::client::CloudStorageError;

/// The main error-handling type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An unexpected status code was received.
    #[error("unexpected status from GCP: {0}")]
    Status(#[from] tonic::Status),
    /// An error with the gRPC transport channel.
    #[error("transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    /// An error with the gRPC metadata value.
    #[error("metadata parse error: {0}")]
    InvalidMetadata(#[from] tonic::metadata::errors::InvalidMetadataValue),
    /// An IO error.
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    /// An environment-related error (missing variable).
    #[error("environment error: {0}")]
    Env(#[from] env::VarError),
    /// Reqwest error (HTTP errors).
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// conversion error (`try_from(..)` or `try_into(..)` errors).
    #[error("conversion error: {0}")]
    Convert(#[from] ConvertError),
    /// authentication-related error.
    #[error("authentication error: {0}")]
    Auth(#[from] AuthError),
    /// url error.
    #[error("url error: {0}")]
    Url(#[from] url::ParseError),
    /// cloud storage API error.
    #[error("cloud storage api error: {0}")]
    CloudStorage(#[from] CloudStorageError),
    /// Google Drive API error.
    #[error("google drive api error: {0}")]
    GooleDrive(#[from] GoogleDriveError),
    /// HeaderValue is not string.
    #[error("HeaderValue is not string: {0}")]
    HeaderValueIsNotString(#[from] ToStrError),
}

/// The error type for value conversions.
#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    /// An expected property was missing.
    #[error("expected property `{0}` was missing")]
    MissingProperty(String),
    /// A value, expected to be an entity, turned out to not be one.
    #[error("expected property type `{expected}`, got `{got}`")]
    UnexpectedPropertyType {
        /// The name of the expected type.
        expected: String,
        /// The name of the actual encountered type.
        got: String,
    },
}

/// The error type for authentication-related errors.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// A JWT-related error.
    #[error("GcpAuth error: {0}")]
    GcpAuth(#[from] gcp_auth::Error),
}
