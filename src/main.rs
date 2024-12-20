use tax_rs::types::{TaxScenario, Region, TransactionType, TaxCalculationType, VatRate};
use tax_rs::provider::TaxDatabase;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}
