use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum InputValidationError {
    #[error("Invalid country code: {0}")]
    InvalidCountryCode(String),
    #[error("Invalid region code: {0}")]
    InvalidRegionCode(String),
    #[error("Unexpected region code: {0} - Country has no regions.")]
    UnexpectedRegionCode(String),
}

#[derive(Debug, Error, Serialize)]
pub enum DatabaseError {
    #[error("Trade agreement not found: {0}")]
    TradeAgreementNotFound(String),
    #[error("Country not found: {0}")]
    CountryNotFound(String),
    // TODO: Implement
    #[error("Region not found: {0}")]
    RegionNotFound(String),
    #[error("VAT rate not found: {0}")]
    VatRateNotFound(String),
}

#[derive(Debug, Error, Serialize)]
pub enum ProcessingError {
    #[error("Invalid input: {0}")]
    InputValidationError(InputValidationError),
    #[error("Database error: {0}")]
    DatabaseError(DatabaseError),
}

impl From<InputValidationError> for ProcessingError {
    fn from(err: InputValidationError) -> Self {
        ProcessingError::InputValidationError(err)
    }
}

impl From<DatabaseError> for ProcessingError {
    fn from(err: DatabaseError) -> Self {
        ProcessingError::DatabaseError(err)
    }
}