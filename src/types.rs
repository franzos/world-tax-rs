use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaxSystemType {
    Vat,
    Gst,
    Pst,
    Hst,
    Qst,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionType {
    B2B,
    B2C,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxCalculationType {
    // Use origin tax rate; below threshold
    Origin,
    // Use destination tax rate; above threshold
    Destination,
    // Buyer pays tax in their country
    ReverseCharge,
    // Tax, but zero-rated
    ZeroRated,
    // No tax
    Exempt,
    // Unknown
    None,
    // Switch to determine threshold-based calculation
    ThresholdBased
}

#[derive(Debug, Clone, Serialize, Deserialize ,PartialEq)]
pub enum TaxType {
    VAT(VatRate),
    GST,
    HST,
    PST,
    StateSalesTax,
    CompoundTax,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VatRate {
    Standard,    
    Reduced,     
    ReducedAlt,  
    SuperReduced,
    Zero,
    Exempt,
    ReverseCharge   
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeAgreementType {
    CustomsUnion, // several countries in a customs union
    FederalState // several states in a country
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAgreement {
    pub name: String,
    pub r#type: TradeAgreementType,
    pub members: Vec<String>,
    pub default_applicable: bool,
    pub applies_to: AppliesTo,
    pub tax_rules: TaxRules,
}

impl TradeAgreement {
    pub fn is_federal(&self) -> bool {
        self.r#type == TradeAgreementType::FederalState
    }

    pub fn is_international(&self) -> bool {
        self.r#type == TradeAgreementType::CustomsUnion
    }
}

#[derive(Debug, Clone)]
pub enum TradeAgreementOverride {
    UseAgreement(String),  // Agreement ID like "EU", "USMCA"
    NoAgreement,           // Force no trade agreement
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliesTo {
    pub physical_goods: bool,
    pub digital_goods: bool,
    pub services: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRuleConfig {
    pub r#type: TaxCalculationType,
    pub below_threshold: Option<TaxCalculationType>,
    pub above_threshold: Option<TaxCalculationType>,
    pub threshold: Option<u32>,
    pub below_threshold_digital_products: Option<TaxCalculationType>,
    pub above_threshold_digital_products: Option<TaxCalculationType>,
    pub threshold_digital_products: Option<u32>,
    pub requires_resale_certificate: Option<bool>,
}

impl TaxRuleConfig {
    pub fn by_threshold(&self, amount: u32, ignore_threshold: bool) -> &TaxCalculationType {
        let has_threshold = self.below_threshold.is_some() && self.above_threshold.is_some() && self.threshold.is_some();
        if has_threshold {
            let rule_threshold: u32 = self.threshold.unwrap();
            if amount < rule_threshold && !ignore_threshold {
                return self.below_threshold.as_ref().unwrap();
            } else {
                return self.above_threshold.as_ref().unwrap();
            }
        }
        &self.r#type
    }

    pub fn by_digital_product_threshold(&self, amount: u32, ignore_threshold: bool) -> &TaxCalculationType {
        let has_threshold = self.below_threshold_digital_products.is_some() && self.above_threshold_digital_products.is_some() && self.threshold_digital_products.is_some();
        if has_threshold {
            let rule_threshold: u32 = self.threshold_digital_products.unwrap();
            if amount < rule_threshold && !ignore_threshold {
                return self.below_threshold_digital_products.as_ref().unwrap();
            } else {
                return self.above_threshold_digital_products.as_ref().unwrap();
            }
        }
        &self.r#type
    }

    pub fn by_threshold_or_digital_product_threshold(&self, amount: u32, is_digital_product_or_service: bool, ignore_threshold: bool) -> &TaxCalculationType {
        if is_digital_product_or_service {
            return self.by_digital_product_threshold(amount, ignore_threshold);
        }
        self.by_threshold(amount, ignore_threshold)
    }

    pub fn is_reseller(&self, has_resale_certificate: bool) -> bool {
        if self.requires_resale_certificate.is_some() {
            return self.requires_resale_certificate.unwrap() && has_resale_certificate;
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRules {
    pub internal_b2b: Option<TaxRuleConfig>,
    pub internal_b2c: Option<TaxRuleConfig>,
    pub external_export: TaxRuleConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRules {
    pub default: String,
    pub specific_products: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub standard_rate: f64,
    #[serde(rename = "type")]
    pub tax_type: TaxSystemType,
}

fn deserialize_rate<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RateValue {
        Number(f64),
        Boolean(bool),
    }

    match Option::<RateValue>::deserialize(deserializer)? {
        Some(RateValue::Number(rate)) => Ok(Some(rate)),
        Some(RateValue::Boolean(false)) => Ok(None),
        Some(RateValue::Boolean(true)) => Ok(None), // Handle true case if needed
        None => Ok(None),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    #[serde(rename = "type")]
    pub tax_type: TaxSystemType,
    pub currency: String,
    pub standard_rate: f64,
    #[serde(default, deserialize_with = "deserialize_rate")]
    pub reduced_rate: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_rate")]
    pub reduced_rate_alt: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_rate")]
    pub super_reduced_rate: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_rate")]
    pub parking_rate: Option<f64>,
    pub vat_name: Option<String>,
    pub vat_abbr: Option<String>,
    pub states: Option<HashMap<String, State>>,
}

#[derive(Debug, Clone)]
pub struct Region {
    pub country: String,
    pub region: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TaxScenario {
    pub source_region: Region,
    pub destination_region: Region,
    pub transaction_type: TransactionType,
    pub calculation_type: TaxCalculationType,
    pub trade_agreement_override: Option<TradeAgreementOverride>,
    pub is_digital_product_or_service: bool,
    // B2B in the US: 0%
    pub has_resale_certificate: bool,
    pub ignore_threshold: bool,
    pub vat_rate: Option<VatRate>,
}
