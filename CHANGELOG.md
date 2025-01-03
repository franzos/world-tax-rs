# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)

## [0.3.1]

### Added

- `calculate_tax_decimal` function to calculate tax for decimal numbers

## [0.3.0]

### Changed

- Bump depdenencies
- More comments to improve usability
- Cleanup exports: All important types are now re-exported in `lib.rs`

### Removed

- `TaxScenario.calculation_type`: Unnecessary
- `TaxType::CompoundTax`: Unnecessary

### Fixed

- Calculation of rates for Quebec (CA), B2C

## [0.2.1]

### Changed

- Improve handling when used as part of an API (snake_case, lowercase serialization)

## [0.2.0]

### Added

- More explicit when something couldn't be found or matched
- Error handling: ProcessingError, DatabaseError

## [0.1.4]

### Changed

- TaxType, VatRate, TaxRate: Macro to Serialize and Deserialize

## [0.1.3]

### Added

- Helper to load database with included rates `TaxDatabase::new()`

## [0.1.2]

### Added

- new `get_rates` function to access the rates directly

### Changed

- Removed unnecessary `main.rs` file
- Refactor tests to satisfy compiler warnings

## [0.1.1]

### Added

- Input validation for countries and regions (states)