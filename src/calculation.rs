use log::debug;

use crate::{errors::InputValidationError, provider::TaxRate, types::{TaxType, TradeAgreement, TradeAgreementOverride, VatRate}};
use super::{provider::TaxDatabase, types::{Region, TaxCalculationType, TaxScenario, TransactionType}};

impl TaxScenario {
    pub fn new(
        source_region: Region,
        destination_region: Region,
        transaction_type: TransactionType,
    ) -> Self {
        Self {
            source_region,
            destination_region,
            transaction_type,
            calculation_type: TaxCalculationType::Destination,
            trade_agreement_override: None,
            is_digital_product_or_service: false,
            has_resale_certificate: false,
            ignore_threshold: false,
            vat_rate: None,
        }
    }

    pub fn with_trade_agreement_override(mut self, override_type: TradeAgreementOverride) -> Self {
        self.trade_agreement_override = Some(override_type);
        self
    }

    pub fn is_same_country(&self) -> bool {
        self.source_region.country == self.destination_region.country
    }

    pub fn is_same_state(&self) -> bool {
        self.source_region.region == self.destination_region.region
    }

    fn get_calculation_from_agreement(&self, agreement: &TradeAgreement, amount: f64) -> TaxCalculationType {
        if agreement.is_international() {
            // Custom union like EU
            match self.transaction_type {
                TransactionType::B2B => {
                    let rule = &agreement.tax_rules.internal_b2b;
                    if rule.is_some() {
                        // In the EU, likely to be reverse charge
                        return rule.clone().unwrap().by_threshold(amount as u32, self.ignore_threshold).clone();
                    } else {
                        // Fallback to a safe option
                        return TaxCalculationType::Destination;
                    }
                },
                TransactionType::B2C => {
                    let rule = &agreement.tax_rules.internal_b2c;
                    if rule.is_some() {
                        // In the EU, by threshold, likely to be origin or destination based
                        return rule.clone().unwrap().by_threshold_or_digital_product_threshold(
                            amount as u32, self.is_digital_product_or_service, self.ignore_threshold
                        ).clone();
                    } else {
                        // Fallback to a safe option
                        return TaxCalculationType::Destination;
                    }
                },
            }
        } else if agreement.is_federal() {
            // States like in the US, CA
            match self.transaction_type {
                TransactionType::B2B => {
                    let rule = &agreement.tax_rules.internal_b2b;
                    if rule.is_some() {
                        let u_rule = rule.clone().unwrap();
                        if u_rule.is_reseller(self.has_resale_certificate) {
                            return TaxCalculationType::ZeroRated;
                        }
                        return rule.clone().unwrap().by_threshold(amount as u32, self.ignore_threshold).clone();
                    } else {
                        // Fallback to a safe option
                        return TaxCalculationType::Destination;
                    }
                },
                TransactionType::B2C => {
                    let rule = &agreement.tax_rules.internal_b2c;
                    if rule.is_some() {
                        // In the US, by threshold, likely to be origin or destination based
                        return  rule.clone().unwrap().by_threshold(amount as u32, self.ignore_threshold).clone();
                    } else {
                        // Fallback to a safe option
                        return TaxCalculationType::Destination;
                    }
                },
            }
        } else {
            return TaxCalculationType::Destination;
        }
    }

    // Figure out which rule to use
    pub fn determine_rule(&self, db: &TaxDatabase) -> Option<TradeAgreement> {
        if self.trade_agreement_override.is_some() {
            let overwrite = self.trade_agreement_override.clone().unwrap();
            match overwrite {
                TradeAgreementOverride::UseAgreement(agreement) => {
                    return db.get_rule(&agreement.as_str());
                },
                TradeAgreementOverride::NoAgreement => {
                    return None;
                }
            }
        }
        if self.is_same_country() {
            // Same country; Federal agreement (for ex. USA)
            db.get_federal_rule(self.source_region.country.as_str())
        } else {
            // Different countries; Customs union agreement (for ex. EU)
            db.get_international_rule(self.source_region.country.as_str(), self.destination_region.country.as_str())
        }
    }

    pub fn determine_calculation_type(&self, db: &TaxDatabase, amount: f64) -> TaxCalculationType {
        // Check if there's a trade rule
        let agreement = self.determine_rule(&db);

        if agreement.is_none() {
            // No agreement found, use default rules
            if self.is_same_country() {
                match self.transaction_type {
                    TransactionType::B2B => {
                        return TaxCalculationType::Origin
                    },
                    TransactionType::B2C => {
                        return TaxCalculationType::Origin
                    }
                }
            } else {
                return TaxCalculationType::ZeroRated
            }
        }

        self.get_calculation_from_agreement(&agreement.unwrap(), amount)
    }

    pub fn get_rates(&self, amount: f64, db: &TaxDatabase) -> Vec<TaxRate> {
        let calculation_type = self.determine_calculation_type(db, amount);

        match calculation_type {
            TaxCalculationType::ReverseCharge => vec![TaxRate {
                tax_type: TaxType::VAT(VatRate::ReverseCharge),
                compound: false,
                rate: 0.0,
            }],
            TaxCalculationType::ZeroRated => vec![TaxRate {
                tax_type: TaxType::VAT(VatRate::Zero),
                compound: false,
                rate: 0.0,
            }], 
            TaxCalculationType::Exempt => vec![TaxRate {
                tax_type: TaxType::VAT(VatRate::Exempt),
                compound: false,
                rate: 0.0,
            }],
            _ => {
                let region = match calculation_type {
                    TaxCalculationType::Origin => &self.source_region,
                    _ => &self.destination_region,
                };

                db.get_rate(
                    &region.country,
                    region.region.as_deref(),
                    self.vat_rate.as_ref()
                )
            }
        }
    }

    pub fn calculate_tax(&self, amount: f64, db: &TaxDatabase) -> f64 {
        let rates = self.get_rates(amount, db);

        let mut total_tax = 0.0;
        let base_amount = amount;

        for rate in rates {
            let tax_amount = if rate.compound {
                (base_amount + total_tax) * rate.rate
            } else {
                base_amount * rate.rate
            };
            total_tax += tax_amount;
        }

        (total_tax * 100.0).round() / 100.0
    }
}

impl Region {
    pub fn new(country: String, region: Option<String>) -> Result<Self, InputValidationError> {
        Self::validate(&country, &region)?;
        Ok(Self { country, region })

    }

    fn validate(country: &str, region: &Option<String>) -> Result<(), InputValidationError> {
        let country_info = rust_iso3166::from_alpha2(country)
            .ok_or_else(|| InputValidationError::InvalidCountryCode(country.to_string()))?;
            
        debug!("Found country: {}", country_info.name);

        if let Some(region_code) = region {
            let _ = country_info.subdivisions()
                .ok_or_else(|| InputValidationError::UnexpectedRegionCode(region_code.clone()))?;
                
            let country_region_code = format!("{}-{}", country, region_code);
            let region = rust_iso3166::iso3166_2::from_code(&country_region_code)
                .ok_or_else(|| InputValidationError::InvalidRegionCode(region_code.clone()))?;

                debug!("Found region: {}", region.name);
        }

        Ok(())
    }
}
