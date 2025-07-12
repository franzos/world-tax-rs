use serde::Serialize;
use typeshare::typeshare;

#[typeshare]
#[derive(Serialize)]
pub struct Country {
    name: String,
    alpha2: String,
}

#[typeshare]
#[derive(Serialize)]
pub struct Subdivision {
    name: String,
    code: String,
}

#[typeshare]
#[derive(Serialize)]
pub struct CountryWithSubdivisions {
    code: String,
    name: String,
    divisions: Vec<Subdivision>,
}

pub fn all_countries() -> Vec<CountryWithSubdivisions> {
    rust_iso3166::countries()
        .map(|country| CountryWithSubdivisions {
            code: country.alpha2,
            name: country.name.to_string(),
            divisions: country
                .subdivisions()
                .map(|subdivision| Subdivision {
                    name: subdivision.name.to_string(),
                    code: subdivision.code.to_string(),
                })
                .collect(),
        })
        .collect()
}
