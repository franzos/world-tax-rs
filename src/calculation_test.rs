#[cfg(test)]
mod tests {
    use crate::types::TaxType;
    use crate::{
        provider::TaxDatabase,
        types::{Region, TransactionType, VatRate, TaxScenario},
    };

    fn setup() -> TaxDatabase {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)  // Set to Debug level
            .try_init();
        TaxDatabase::from_files("vat_rates.json", "trade_agreements.json").expect("Tax database should load")
    }

    #[test]
    fn test_german_vat_calculation() {
        let db = setup();
        let scenario = TaxScenario::new(
            Region::new("DE".to_string(), None).expect("Valid German region"),
            Region::new("DE".to_string(), None).expect("Valid German region"),
            TransactionType::B2C,
        );
        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 19.0); // Germany's actual VAT rate

        let rates = scenario.get_rates(100.0, &db).expect("Rates should be found");
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].rate, 0.19);
        assert_eq!(rates[0].tax_type, TaxType::VAT(VatRate::Standard));
        assert_eq!(rates[0].compound, false);
    }

    #[test]
    fn test_canadian_gst_bc_pst_below_threshold() {
        let db = setup();
        let scenario = TaxScenario::new(
            Region::new("CA".to_string(), Some("BC".to_string())).expect("Valid Canadian BC region"),
            Region::new("CA".to_string(), Some("BC".to_string())).expect("Valid Canadian BC region"),
            TransactionType::B2C,
        );

        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 0.0); // Combined GST (5%) + PST (7%) for British Columbia
    }

    #[test]
    fn test_canadian_gst_bc_pst_ignore_threshold() {
        let db = setup();
        let mut scenario = TaxScenario::new(
            Region::new("CA".to_string(), Some("BC".to_string())).expect("Valid Canadian BC region"),
            Region::new("CA".to_string(), Some("BC".to_string())).expect("Valid Canadian BC region"),
            TransactionType::B2C,
        );
        scenario.ignore_threshold = true;

        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 12.35); // Combined GST (5%) + PST (7%) for British Columbia
    }

    #[test]
    fn test_canadian_gst_bc_pst_above_threshold() {
        let db = setup();
        let scenario = TaxScenario::new(
            Region::new("CA".to_string(), Some("BC".to_string())).expect("Valid Canadian BC region"),
            Region::new("CA".to_string(), Some("BC".to_string())).expect("Valid Canadian BC region"),
            TransactionType::B2C,
        );

        let tax = scenario.calculate_tax(100000.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 12350.0); // Combined GST (5%) + PST (7%) for British Columbia
    }

    #[test]
    fn test_eu_cross_border_b2b() {
        let db = setup();
        let scenario = TaxScenario::new(
            Region::new("DE".to_string(), None).expect("Valid German region"),
            Region::new("FR".to_string(), None).expect("Valid French region"),
            TransactionType::B2B,
        );

        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 0.0); // EU reverse charge mechanism
    }

    #[test]
    fn test_eu_cross_border_b2c_digital() {
        let db = setup();
        let mut scenario = TaxScenario::new(
            Region::new("DE".to_string(), None).expect("Valid German region"),
            Region::new("FR".to_string(), None).expect("Valid French region"),
            TransactionType::B2B,
        );
        scenario.is_digital_product_or_service = true;

        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 0.0); // EU reverse charge mechanism
    }

    #[test]
    fn test_eu_cross_border_b2c() {
        let db = setup();
        let scenario = TaxScenario::new(
            Region::new("DE".to_string(), None).expect("Valid German region"),
            Region::new("FR".to_string(), None).expect("Valid French region"),
            TransactionType::B2C,
        );

        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 19.0); // EU reverse charge mechanism
    }

    #[test]
    fn test_french_reduced_vat() {
        let db = setup();
        let mut scenario = TaxScenario::new(
            Region::new("FR".to_string(), None).expect("Valid French region"),
            Region::new("FR".to_string(), None).expect("Valid French region"),
            TransactionType::B2C,
        );
        scenario.vat_rate = Some(VatRate::ReducedAlt);

        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 5.5); // France's actual reduced VAT rate
    }

    #[test]
    fn test_german_domestic_b2b() {
        let db = setup();
        let scenario = TaxScenario::new(
            Region::new("DE".to_string(), None).expect("Valid German region"),
            Region::new("DE".to_string(), None).expect("Valid German region"),
            TransactionType::B2B,
        );

        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 19.0); // German domestic B2B VAT rate
    }

    #[test]
    fn test_germany_thailand_b2b() {
        let db = setup();
        let scenario = TaxScenario::new(
            Region::new("DE".to_string(), None).expect("Valid German region"),
            Region::new("TH".to_string(), None).expect("Valid Thai region"),
            TransactionType::B2B,
        );

        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 0.0); // Export from EU is zero-rated
    }

    #[test]
    fn test_germany_thailand_b2c() {
        let db = setup();
        let scenario = TaxScenario::new(
            Region::new("DE".to_string(), None).expect("Valid German region"),
            Region::new("TH".to_string(), None).expect("Valid Thai region"),
            TransactionType::B2C,
        );

        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 0.0); // Export from EU to non-EU country is zero-rated for B2C too
    }

    #[test]
    fn test_us_interstate_b2c_below_threshold() {
        let db = setup();
        let scenario = TaxScenario::new(
            Region::new("US".to_string(), Some("CA".to_string())).expect("Valid US-CA region"),
            Region::new("US".to_string(), Some("WA".to_string())).expect("Valid US-WA region"),
            TransactionType::B2C,
        );

        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 0.0); // Washington state sales tax rate for remote sellers
    }

    #[test]
    fn test_us_interstate_b2c_ignore_threshold() {
        let db = setup();
        let mut scenario = TaxScenario::new(
            Region::new("US".to_string(), Some("CA".to_string())).expect("Valid US-CA region"),
            Region::new("US".to_string(), Some("WA".to_string())).expect("Valid US-WA region"),
            TransactionType::B2C,
        );
        scenario.ignore_threshold = true;

        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 6.5); // Washington state sales tax rate for remote sellers
    }

    #[test]
    fn test_us_interstate_b2c_above_threshold() {
        let db = setup();
        let scenario = TaxScenario::new(
            Region::new("US".to_string(), Some("CA".to_string())).expect("Valid US-CA region"),
            Region::new("US".to_string(), Some("WA".to_string())).expect("Valid US-WA region"),
            TransactionType::B2C,
        );

        let tax = scenario.calculate_tax(100000.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 6500.0); // Washington state sales tax rate for remote sellers
    }

    #[test]
    fn test_us_interstate_b2b() {
        let db = setup();
        let mut scenario = TaxScenario::new(
            Region::new("US".to_string(), Some("TX".to_string())).expect("Valid US-TX region"),
            Region::new("US".to_string(), Some("WA".to_string())).expect("Valid US-WA region"),
            TransactionType::B2B,
        );
        scenario.has_resale_certificate = true;

        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 0.0);
    }

    #[test]
    fn test_us_interstate_b2b_reseller() {
        let db = setup();
        let mut scenario = TaxScenario::new(
            Region::new("US".to_string(), Some("WA".to_string())).expect("Valid US-WA region"),
            Region::new("US".to_string(), Some("TX".to_string())).expect("Valid US-TX region"),
            TransactionType::B2B,
        );
        scenario.has_resale_certificate = true;

        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 0.0);
    }

    #[test]
    fn test_gcc_cross_border_b2b() {
        let db = setup();
        let scenario = TaxScenario::new(
            Region::new("AE".to_string(), None).expect("Valid UAE region"),
            Region::new("QA".to_string(), None).expect("Valid Qatar region"),
            TransactionType::B2B,
        );

        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 0.0); // GCC countries have no VAT
    }

    #[test]
    fn test_gcc_cross_border_b2c() {
        let db = setup();
        let scenario = TaxScenario::new(
            Region::new("AE".to_string(), None).expect("Valid UAE region"),
            Region::new("QA".to_string(), None).expect("Valid Qatar region"),
            TransactionType::B2C,
        );

        let tax = scenario.calculate_tax(100.0, &db).expect("Tax calculation should succeed");
        assert_eq!(tax, 5.0); // GCC countries have no VAT
    }

    #[test]
    fn load_included_db() {
        let _ = TaxDatabase::new();
    }
}