#![allow(dead_code)]

use std::collections::BTreeMap;
use struct_deep_getter::StructDeepGetter;
use struct_deep_getter_derive::make_paths;

struct SuperType {
    value: String
}

impl From<Option<&str>> for SuperType {
    fn from(s: Option<&str>) -> Self {
        SuperType { value: s.unwrap_or("None").to_owned() }
    }
}

impl From<Option<u16>> for SuperType {
    fn from(s: Option<u16>) -> Self {
        SuperType { value: format!("{}", s.unwrap_or(0)) }
    }
}

impl From<Option<u32>> for SuperType {
    fn from(s: Option<u32>) -> Self {
        SuperType { value: format!("{}", s.unwrap_or(0)) }
    }
}

impl From<Option<f64>> for SuperType {
    fn from(s: Option<f64>) -> Self {
        SuperType { value: format!("{}", s.unwrap_or(0.0)) }
    }
}

impl From<Option<bool>> for SuperType {
    fn from(s: Option<bool>) -> Self {
        SuperType { value: format!("{}", s.unwrap_or(false)) }
    }
}

pub struct MaxmindCity<'a> {
    pub city: Option<City2<'a>>,
    pub continent: Option<Continent<'a>>,
    pub country: Option<Country<'a>>,
    pub location: Option<Location<'a>>,
    pub postal: Option<Postal<'a>>,
    pub registered_country: Option<Country<'a>>,
    pub represented_country: Option<RepresentedCountry<'a>>,
    pub subdivisions: Option<Vec<Subdivision<'a>>>,
    pub traits: Option<Traits>,
}

pub struct City2<'a> {
    pub geoname_id: Option<u32>,
    pub names: Option<BTreeMap<&'a str, &'a str>>,
}

pub struct Location<'a> {
    pub accuracy_radius: Option<u16>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub metro_code: Option<u16>,
    pub time_zone: Option<&'a str>,
}

pub struct Postal<'a> {
    pub code: Option<&'a str>,
}

pub struct Subdivision<'a> {
    pub geoname_id: Option<u32>,
    pub iso_code: Option<&'a str>,
    pub names: Option<BTreeMap<&'a str, &'a str>>,
}

pub struct Continent<'a> {
    pub code: Option<&'a str>,
    pub geoname_id: Option<u32>,
    pub names: Option<BTreeMap<&'a str, &'a str>>,
}

pub struct Country<'a> {
    pub geoname_id: Option<u32>,
    pub is_in_european_union: Option<bool>,
    pub iso_code: Option<&'a str>,
    pub names: Option<BTreeMap<&'a str, &'a str>>,
}

pub struct RepresentedCountry<'a> {
    pub geoname_id: Option<u32>,
    pub is_in_european_union: Option<bool>,
    pub iso_code: Option<&'a str>,
    pub names: Option<BTreeMap<&'a str, &'a str>>,
    pub representation_type: Option<&'a str>,
}

pub struct Traits {
    pub is_anonymous_proxy: Option<bool>,
    pub is_satellite_provider: Option<bool>,
}

make_paths!(
    #[struct_deep_getter(return_type = "SuperType", replacement_type = "MaxmindCity")]
    pub struct City<'a> {
        pub city: Option<City2<'a>>,
        pub continent: Option<Continent<'a>>,
        pub country: Option<Country<'a>>,
        pub location: Option<Location<'a>>,
        pub postal: Option<Postal<'a>>,
        pub registered_country: Option<Country<'a>>,
        pub represented_country: Option<RepresentedCountry<'a>>,
        pub subdivisions: Option<Vec<Subdivision<'a>>>,
        pub traits: Option<Traits>,
    }

    pub struct City2<'a> {
        pub geoname_id: Option<u32>,
        pub names: Option<BTreeMap<&'a str, &'a str>>,
    }

    pub struct Location<'a> {
        pub accuracy_radius: Option<u16>,
        pub latitude: Option<f64>,
        pub longitude: Option<f64>,
        pub metro_code: Option<u16>,
        pub time_zone: Option<&'a str>,
    }

    pub struct Postal<'a> {
        pub code: Option<&'a str>,
    }

    pub struct Subdivision<'a> {
        pub geoname_id: Option<u32>,
        pub iso_code: Option<&'a str>,
        pub names: Option<BTreeMap<&'a str, &'a str>>,
    }

    pub struct Continent<'a> {
        pub code: Option<&'a str>,
        pub geoname_id: Option<u32>,
        pub names: Option<BTreeMap<&'a str, &'a str>>,
    }

    pub struct Country<'a> {
        pub geoname_id: Option<u32>,
        pub is_in_european_union: Option<bool>,
        pub iso_code: Option<&'a str>,
        pub names: Option<BTreeMap<&'a str, &'a str>>,
    }

    pub struct RepresentedCountry<'a> {
        pub geoname_id: Option<u32>,
        pub is_in_european_union: Option<bool>,
        pub iso_code: Option<&'a str>,
        pub names: Option<BTreeMap<&'a str, &'a str>>,
        pub representation_type: Option<&'a str>,
    }

    pub struct Traits {
        pub is_anonymous_proxy: Option<bool>,
        pub is_satellite_provider: Option<bool>,
    }
);

#[test]
fn test_paths() {
    assert_eq!(MaxmindCity::deeper_structs(), vec!["lol"])
}
