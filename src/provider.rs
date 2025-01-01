use std::collections::HashMap;
use crate::types::TradeAgreement;
use super::types::{Country, TaxSystemType, TaxType, VatRate};

#[derive(Debug)]
pub struct TaxRate {
    pub rate: f64,
    pub tax_type: TaxType,
    pub compound: bool,
}

pub struct TaxDatabase {
    countries: HashMap<String, Country>,
    pub trade_agreements: HashMap<String, TradeAgreement>,
}

impl TaxDatabase {
    pub fn from_json(countries_json: &str, trade_agreements_json: &str) -> Result<Self, serde_json::Error> {
        let countries: HashMap<String, Country> = serde_json::from_str(countries_json)?;
        let trade_agreements: HashMap<String, TradeAgreement> = serde_json::from_str(trade_agreements_json)?;
        Ok(Self { countries, trade_agreements })
    }

    pub fn from_files(rates_path: &str, agreements_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let rates_data = std::fs::read_to_string(rates_path)?;
        let agreements_data = std::fs::read_to_string(agreements_path)?;
        
        let countries = serde_json::from_str(&rates_data)?;
        let trade_agreements = serde_json::from_str(&agreements_data)?;
        
        Ok(Self { countries, trade_agreements })
    }

    pub fn get_federal_rule(&self, country: &str) -> Option<TradeAgreement> {
        let rule = self.trade_agreements.get(country);
        if rule.is_some() && rule.unwrap().is_federal() {
            Some(rule.unwrap().clone())
        } else {
            None
        }
    }

    pub fn get_international_rule(&self, source: &str, dest: &str) -> Option<TradeAgreement> {
        for agreement in self.trade_agreements.values() {
            if agreement.members.contains(&source.to_string()) && 
            agreement.members.contains(&dest.to_string()) && 
            agreement.is_international() {
            return Some(agreement.clone());
            }
        }
        None
    }

    pub fn get_rule(&self, rule_id: &str) -> Option<TradeAgreement> {
        let rule = self.trade_agreements.get(rule_id);
        if rule.is_some() {
            Some(rule.unwrap().clone())
        } else {
            None
        }
    }

    pub fn get_country(&self, code: &str) -> Option<&Country> {
        self.countries.get(code)
    }

    pub fn get_rate(&self, country: &str, region: Option<&str>, vat_rate: Option<&VatRate>) -> Vec<TaxRate> {
        let mut rates = Vec::new();
        
        if let Some(country_data) = self.countries.get(country) {
            if country == "US" && region.is_some() {
                if let Some(states) = &country_data.states {
                    if let Some(state) = states.get(region.unwrap()) {
                        rates.push(TaxRate {
                            rate: state.standard_rate,
                            tax_type: TaxType::StateSalesTax,
                            compound: false,
                        });
                    }
                }
            }
            match country_data.tax_type {
                TaxSystemType::Vat => {
                    if let Some(rate) = match vat_rate.unwrap_or(&VatRate::Standard) {
                        VatRate::Standard => Some(country_data.standard_rate),
                        VatRate::Reduced => country_data.reduced_rate,
                        VatRate::ReducedAlt => country_data.reduced_rate_alt,
                        VatRate::SuperReduced => country_data.super_reduced_rate,
                        VatRate::Zero => Some(0.0),
                        VatRate::Exempt => Some(0.0),
                        VatRate::ReverseCharge => Some(0.0),
                    } {
                        rates.push(TaxRate {
                            rate,
                            tax_type: TaxType::VAT(vat_rate.cloned().unwrap_or(VatRate::Standard)),
                            compound: false,
                        });
                    }
                },
                TaxSystemType::Gst => {
                    rates.push(TaxRate {
                        rate: country_data.standard_rate,
                        tax_type: TaxType::GST,
                        compound: false,
                    });

                    if let Some(states) = &country_data.states {
                        if let Some(region) = region {
                            if let Some(state) = states.get(region) {
                                let tax_type = match state.tax_type {
                                    TaxSystemType::Pst => TaxType::PST,
                                    TaxSystemType::Hst => TaxType::HST,
                                    _ => return rates,
                                };
                                rates.push(TaxRate {
                                    rate: state.standard_rate,
                                    tax_type: tax_type.clone(),
                                    compound: matches!(tax_type, TaxType::PST),
                                });
                            }
                        }
                    }
                },
                _ => {}
            }
        }
        
        rates
    }
}
