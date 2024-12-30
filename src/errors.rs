use thiserror::Error;

#[derive(Debug, Error)]
pub enum InputValidationError {
    #[error("Invalid country code: {0}")]
    InvalidCountryCode(String),
    #[error("Invalid region code: {0}")]
    InvalidRegionCode(String),
    #[error("Unexpected region code: {0} - Country has no regions.")]
    UnexpectedRegionCode(String),
}