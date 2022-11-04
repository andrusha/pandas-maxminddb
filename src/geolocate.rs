use crate::lookup_result::LookupResult;
use crate::{GeoColumn, PandasMaxmindError};
use maxminddb::{geoip2, Reader};
use rayon::prelude::*;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;

/*
   Treats missing lookup as non-critical error
   in order to short-circuit execution down the line
*/
fn lookup_ip<'py, T: AsRef<[u8]>>(
    ip: &str,
    reader: &'py Reader<T>,
) -> Result<Option<geoip2::City<'py>>, PandasMaxmindError> {
    let ip = ip.parse::<IpAddr>();

    match ip {
        Ok(ip) => match reader.lookup(ip) {
            Ok(l) => Ok(Some(l)),
            Err(maxminddb::MaxMindDBError::AddressNotFoundError(_)) => Ok(None),
            Err(e) => Err(e.into()),
        },
        Err(_) => Ok(None),
    }
}

#[cfg(test)]
mod test_lookup_ip {
    use super::lookup_ip;
    use maxminddb::Reader;

    #[test]
    fn ignores_wrong_ip() {
        let reader = Reader::open_mmap("./GeoLite.mmdb/GeoLite2-City.mmdb").unwrap();
        let res = lookup_ip("gibberish", &reader);
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());
    }

    #[test]
    fn ignores_missed_lookup() {
        let reader = Reader::open_mmap("./GeoLite.mmdb/GeoLite2-City.mmdb").unwrap();
        let res = lookup_ip("255.255.255.255", &reader);
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());
    }

    #[test]
    fn returns_result() {
        let reader = Reader::open_mmap("./GeoLite.mmdb/GeoLite2-City.mmdb").unwrap();
        let res = lookup_ip("75.63.106.74", &reader);
        assert!(res.is_ok());
        assert!(res.unwrap().is_some());
    }
}

/*
   Splits the slice into chunks, each then geolocated in parallel,
   and result is unioned together.
   Optimal chunk size depends on the dataset and the platform
*/
pub fn geolocate_par<'r>(
    ips: &[String],
    reader: &'r Reader<Vec<u8>>,
    columns: &[GeoColumn],
    chunk_size: usize,
) -> Result<HashMap<GeoColumn, Vec<LookupResult<'r>>>, PandasMaxmindError> {
    let reader_arc = Arc::new(reader);
    let chunks: Vec<Result<HashMap<GeoColumn, Vec<LookupResult<'r>>>, PandasMaxmindError>> = ips
        .par_chunks(chunk_size)
        .map(|chunk| geolocate(chunk, &reader_arc, columns))
        .collect();

    let mut res = HashMap::with_capacity(columns.len());
    for &c in columns.iter() {
        res.insert(c, Vec::with_capacity(ips.len()));
    }

    // Union chunks together
    for chunk in chunks {
        for (k, v) in chunk?.iter_mut() {
            res.get_mut(k).unwrap().append(v);
        }
    }

    Ok(res)
}

/*
   Geolocates given slice with any reader
*/
pub fn geolocate<'r, T: AsRef<[u8]>>(
    ips: &[String],
    reader: &'r Reader<T>,
    columns: &[GeoColumn],
) -> Result<HashMap<GeoColumn, Vec<LookupResult<'r>>>, PandasMaxmindError> {
    let mut res = HashMap::with_capacity(columns.len());
    for &c in columns.iter() {
        res.insert(c, Vec::with_capacity(ips.len()));
    }

    for ip in ips {
        let lookup: Option<geoip2::City> = lookup_ip(ip, reader)?;

        for (col, vec) in res.iter_mut() {
            let v = match col {
                GeoColumn::Country => lookup
                    .as_ref()
                    .and_then(|l| l.country.as_ref())
                    .and_then(|c| c.iso_code)
                    .into(),
                GeoColumn::State => lookup
                    .as_ref()
                    .and_then(|l| l.subdivisions.as_ref())
                    .and_then(|sd| sd.first())
                    .and_then(|s| s.iso_code)
                    .into(),
                GeoColumn::City => lookup
                    .as_ref()
                    .and_then(|l| l.city.as_ref())
                    .and_then(|c| c.names.as_ref())
                    .and_then(|n| n.get("en").copied())
                    .into(),
                GeoColumn::Postcode => lookup
                    .as_ref()
                    .and_then(|l| l.postal.as_ref())
                    .and_then(|c| c.code)
                    .into(),
                GeoColumn::Longitude => lookup
                    .as_ref()
                    .and_then(|l| l.location.as_ref())
                    .and_then(|l| l.longitude)
                    .into(),
                GeoColumn::Latitude => lookup
                    .as_ref()
                    .and_then(|l| l.location.as_ref())
                    .and_then(|l| l.latitude)
                    .into(),
                GeoColumn::AccuracyRadius => lookup
                    .as_ref()
                    .and_then(|l| l.location.as_ref())
                    .and_then(|l| l.accuracy_radius)
                    .into(),
            };

            vec.push(v);
        }
    }

    Ok(res)
}

#[cfg(test)]
mod test_geolocate {
    use super::geolocate;
    use crate::geolocate::geolocate_par;
    use crate::{GeoColumn, LookupResult};
    use maxminddb::Reader;

    #[test]
    fn empty_array() {
        let reader = Reader::open_mmap("./GeoLite.mmdb/GeoLite2-City.mmdb").unwrap();

        let ips = Vec::new();
        let res = geolocate(&ips, &reader, &[GeoColumn::City]);
        assert!(res.is_ok());
        let hm = res.unwrap();
        assert!(hm.contains_key(&GeoColumn::City));
        assert!(hm[&GeoColumn::City].is_empty());
    }

    fn unwrap_strings<'a>(xs: &'a [LookupResult]) -> Vec<Option<&'a str>> {
        xs.iter()
            .map(|c| match c {
                LookupResult::String(s) => *s,
                _ => panic!(),
            })
            .collect()
    }

    fn unwrap_floats(xs: &[LookupResult]) -> Vec<Option<f64>> {
        xs.iter()
            .map(|c| match c {
                LookupResult::Float(f) => *f,
                _ => panic!(),
            })
            .collect()
    }

    #[test]
    fn geolocates() {
        let reader = Reader::open_mmap("./GeoLite.mmdb/GeoLite2-City.mmdb").unwrap();
        let ips: Vec<String> = vec![
            "75.63.106.74",
            "255.255.255.255",
            "gibberish",
            "132.206.246.203",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect();

        let res = geolocate(&ips, &reader, &[GeoColumn::City, GeoColumn::Latitude]);
        assert!(res.is_ok());
        let hm = res.unwrap();
        assert!(hm.contains_key(&GeoColumn::City));
        assert!(hm.contains_key(&GeoColumn::Latitude));

        assert_eq!(
            unwrap_strings(&hm[&GeoColumn::City]),
            vec![Some("Houston"), None, None, Some("Montreal"),]
        );
        assert_eq!(
            unwrap_floats(&hm[&GeoColumn::Latitude]),
            vec![Some(29.9787), None, None, Some(45.5063)]
        );
    }

    #[test]
    fn geolocates_in_parallel() {
        let reader = Reader::open_readfile("./GeoLite.mmdb/GeoLite2-City.mmdb").unwrap();
        let ips: Vec<String> = vec![
            "75.63.106.74",
            "255.255.255.255",
            "gibberish",
            "132.206.246.203",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect();

        // Set chunks to 1 to actually trigger parallelism
        let res = geolocate_par(&ips, &reader, &[GeoColumn::City, GeoColumn::Latitude], 1);
        assert!(res.is_ok());
        let hm = res.unwrap();
        assert!(hm.contains_key(&GeoColumn::City));
        assert!(hm.contains_key(&GeoColumn::Latitude));

        assert_eq!(
            unwrap_strings(&hm[&GeoColumn::City]),
            vec![Some("Houston"), None, None, Some("Montreal"),]
        );
        assert_eq!(
            unwrap_floats(&hm[&GeoColumn::Latitude]),
            vec![Some(29.9787), None, None, Some(45.5063)]
        );
    }
}
