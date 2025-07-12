# Calculate TAX Rates Worldwide

This is meant to be used to calculate B2B/B2C rates on sales.

In theory, all countries are supported, and rules are in place to handle some special cases, including sales within, and from the EU, between GCC countries, Canadian provinces and US states.

## Warning

I cannot guarantee that this is 100% accurate, nor up to date.

Best practice:

- Check the rates and trade agreements (JSON)
- Run tests for your specific use case
- Ask your accountant

If something is off, I'd appreciate a PR.

## Usage

Quick preview:

```rs
// Load the database from included JSON files
let db = TaxDatabase::new()?;

// Load the database from your own JSON files
let rates_json_data = include_str!("../vat_rates.json");
let agreements_json_data = include_str!("../trade_agreements.json");
let db = TaxDatabase::from_json(rates_json_data, agreements_json_data)?;


// German B2C scenario
let scenario = TaxScenario::new(
    Region::new("DE".to_string(), None).expect("Valid German region"),
    Region::new("DE".to_string(), None).expect("Valid German region"),
    TransactionType::B2C,
);
let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
assert_eq!(tax, 19.0); // Germany's MWST

let rates = scenario.get_rates(100.0, &db).expect("Rates should be available");
assert_eq!(rates.len(), 1);
assert_eq!(rates[0].rate, 0.19);
assert_eq!(rates[0].tax_type, TaxType::VAT(VatRate::Standard));
assert_eq!(rates[0].compound, false);


// EU B2B scenario
let scenario = TaxScenario::new(
    Region::new("DE".to_string(), None).expect("Valid German region"),
    Region::new("FR".to_string(), None).expect("Valid French region"),
    TransactionType::B2B,
);

let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
assert_eq!(tax, 0.0); // EU reverse charge mechanism


// EU export
let scenario = TaxScenario::new(
    Region::new("DE".to_string(), None).expect("Valid German region"),
    Region::new("TH".to_string(), None).expect("Valid Thai region"),
    TransactionType::B2C,
);

let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
assert_eq!(tax, 0.0); // Export from EU to non-EU country is zero-rated for B2C too


// USA B2C scenario; ignore threshold
let mut scenario = TaxScenario::new(
    Region::new("US".to_string(), Some("US-CA".to_string())).expect("Valid US-CA region"),
    Region::new("US".to_string(), Some("US-WA".to_string())).expect("Valid US-WA region"),
    TransactionType::B2C,
);
scenario.ignore_threshold = true;

let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
assert_eq!(tax, 6.5); // Washington state sales tax rate for remote sellers


// Canadian B2C scenario; above threshold
let scenario = TaxScenario::new(
    Region::new("CA".to_string(), Some("CA-BC".to_string())).expect("Valid Canadian BC region"),
    Region::new("CA".to_string(), Some("CA-BC".to_string())).expect("Valid Canadian BC region"),
    TransactionType::B2C,
);

let tax = scenario.calculate_tax(100000.0, &db).expect("Tax calculation should succeed");
assert_eq!(tax, 12350.0); // Combined GST (5%) + PST (7%) for British Columbia


// Canadian B2C: More options
let ca_domestic = TaxScenario {
    source_region: Region::new("CA".to_string(), Some("CA-BC".to_string())).expect("Country and region code is invalid"),
    destination_region: Region::new("CA".to_string(), Some("CA-BC".to_string())).expect("Country and region code is invalid"),
    transaction_type: TransactionType::B2C,
    trade_agreement_override: None,
    is_digital_product_or_service: false,
    has_resale_certificate: false,
    ignore_threshold: false,
    vat_rate: None,
};

println!("CA domestic tax: {}", ca_domestic.calculate_tax(100.0, &db));
```

Refer to the tests for more examples.

### Calculation type

Options are:
- `TaxCalculationType::Origin`: Calculated based on the source
- `TaxCalculationType::Destination`: Calculated based on the destination
- `TaxCalculationType::ReverseCharge`: Commonly found in EU B2B transactions
- `TaxCalculationType::ZeroRated`: No tax
- `TaxCalculationType::Exempt`: Tax is exempt
- `TaxCalculationType::None`: No calculation

Lastly, this is mostly for internal use:
- `TaxCalculationType::ThresholdBased`

#### EU-Example

For example, in the EU there's a 10,000 Euro threshold for B2C transactions. If the threshold is exceeded, the calculation type changes from `TaxCalculationType::Origin` to `TaxCalculationType::Destination`.

For digital goods (`is_digital_product_or_service`), the treshold is 0 Euro.

### Trade agreements

Trade agreements are selected automatically, but you may override them by providing a `trade_agreement_override` in the `TaxScenario`.

Options are:

- `TradeAgreementOverride::UseAgreement(String)`
- `TradeAgreementOverride::NoAgreement`

Valid trade agreements are:

- `EU` (customs union)
- `GCC` (customs union)
- `US` (federal state)
- `CA` (federal state)

There's no input validation at the moment.

### Frontend

If you have a JS/TS frontend, you can use [rust_iso3166-ts](https://github.com/franzos/rust_iso3166-ts) to access countries and subdivisions provided by `rust_iso3166` to make sure your inputs are identical.

### TypeScript Types

Generate TypeScript types using [typeshare](https://1password.github.io/typeshare/):

```bash
cargo install typeshare-cli
typeshare . --lang=typescript --output-file=types.ts
```

## Update Rates

```bash
guix shell python3 -- python3 get_vat_rates.py
```

## Test

```bash
RUST_LOG=debug cargo test -- --nocapture
RUST_LOG=debug cargo test -- --test-threads=1 --nocapture
```

## Credit

The tax rates are updated from the following sources:
- https://github.com/valeriansaliou/node-sales-tax/blob/master/res/sales_tax_rates.json
- https://github.com/benbucksch/eu-vat-rates/blob/master/rates.json

Countries and states input is validated using:
- https://github.com/rust-iso/rust_iso3166


