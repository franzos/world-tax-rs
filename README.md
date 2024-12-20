# Calculate TAX Rates Worldwide

This is meant to be used to calculate B2B/B2C rates on sales.

In theory, all countries are supported, and rules are in place to handle some special cases, including sales within, and from the EU, between GCC countries, Canadian provinces and US states.

## Warning

I cannot guarantee that this is 100% accurate, nor up to date.

Best practice:

- Check the rates and trade agreements (JSON)
- Run tests for your specific use case

If something is off, I'd appreciate a PR.

## Usage

Quick preview:

```rs
let rates_json_data = include_str!("../vat_rates.json");
let agreements_json_data = include_str!("../trade_agreements.json");
let db = TaxDatabase::from_json(rates_json_data, agreements_json_data)?;

// EU B2B scenario
let eu_b2b = TaxScenario {
    source_region: Region::new("DE".to_string(), None),
    destination_region: Region::new("FR".to_string(), None),
    transaction_type: TransactionType::B2B,
    calculation_type: TaxCalculationType::Destination,
    trade_agreement_override: None,
    is_digital_product_or_service: false,
    has_resale_certificate: false,
    ignore_threshold: false,
    vat_rate: Some(VatRate::Standard),
};

// Canadian domestic scenario
let ca_domestic = TaxScenario {
    source_region: Region::new("CA".to_string(), Some("BC".to_string())),
    destination_region: Region::new("CA".to_string(), Some("BC".to_string())),
    transaction_type: TransactionType::B2C,
    calculation_type: TaxCalculationType::Destination,
    trade_agreement_override: None,
    is_digital_product_or_service: false,
    has_resale_certificate: false,
    ignore_threshold: false,
    vat_rate: None,
};

let amount = 100.0;
println!("EU B2B tax: {}", eu_b2b.calculate_tax(amount, &db));
println!("CA domestic tax: {}", ca_domestic.calculate_tax(amount, &db));
```

Refer to the tests for more examples.

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

The VAT rates are updated from the following sources:
- https://github.com/valeriansaliou/node-sales-tax/blob/master/res/sales_tax_rates.json
- https://github.com/benbucksch/eu-vat-rates/blob/master/rates.json


