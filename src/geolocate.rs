use crate::{GeoColumn, PandasMaxmindError};
use maxminddb::{geoip2, Reader};
use numpy::PyReadonlyArray1;
use pyo3::{PyObject, Python, ToPyObject};
use std::collections::HashMap;
use std::net::IpAddr;

// Treats missing lookup as non-critical error
// in order to short-circuit execution down the line
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

pub fn geolocate<'py, T: AsRef<[u8]>>(
    py: Python<'py>,
    ips: PyReadonlyArray1<PyObject>,
    reader: &Reader<T>,
    columns: Vec<GeoColumn>,
) -> Result<HashMap<GeoColumn, Vec<PyObject>>, PandasMaxmindError> {
    let mut res = HashMap::with_capacity(columns.len());
    for &c in columns.iter() {
        res.insert(c, Vec::with_capacity(ips.len()));
    }

    for ip in ips.as_array().iter() {
        let lookup: Option<geoip2::City> = lookup_ip(&ip.to_string(), reader)?;

        for (col, vec) in res.iter_mut() {
            let v = match col {
                GeoColumn::Country => lookup
                    .as_ref()
                    .and_then(|l| l.country.as_ref())
                    .and_then(|c| c.iso_code)
                    .to_object(py),

                GeoColumn::State => lookup
                    .as_ref()
                    .and_then(|l| l.subdivisions.as_ref())
                    .and_then(|sd| sd.first())
                    .and_then(|s| s.iso_code)
                    .to_object(py),

                GeoColumn::City => lookup
                    .as_ref()
                    .and_then(|l| l.city.as_ref())
                    .and_then(|c| c.names.as_ref())
                    .and_then(|n| n.get("en"))
                    .to_object(py),

                GeoColumn::Postcode => lookup
                    .as_ref()
                    .and_then(|l| l.postal.as_ref())
                    .and_then(|c| c.code)
                    .to_object(py),

                GeoColumn::Longitude => lookup
                    .as_ref()
                    .and_then(|l| l.location.as_ref())
                    .and_then(|l| l.longitude)
                    .to_object(py),

                GeoColumn::Latitude => lookup
                    .as_ref()
                    .and_then(|l| l.location.as_ref())
                    .and_then(|l| l.latitude)
                    .to_object(py),

                GeoColumn::AccuracyRadius => lookup
                    .as_ref()
                    .and_then(|l| l.location.as_ref())
                    .and_then(|l| l.accuracy_radius)
                    .to_object(py),
            };

            vec.push(v);
        }
    }

    Ok(res)
}

#[cfg(test)]
mod test_geolocate {
    use super::geolocate;
    use crate::GeoColumn;
    use maxminddb::Reader;
    use numpy::{PyArray1, PyReadonlyArray1};
    use pyo3::{PyObject, Python, ToPyObject};

    #[test]
    fn empty_ndarray() {
        let reader = Reader::open_mmap("./GeoLite.mmdb/GeoLite2-City.mmdb").unwrap();

        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let ips: PyReadonlyArray1<PyObject> = PyArray1::from_vec(py, Vec::new()).readonly();
            let res = geolocate(py, ips, &reader, vec![GeoColumn::City]);
            assert!(res.is_ok());
            let hm = res.unwrap();
            assert!(hm.contains_key(&GeoColumn::City));
            assert!(hm[&GeoColumn::City].is_empty());
        });
    }

    #[test]
    fn geolocates() {
        let reader = Reader::open_mmap("./GeoLite.mmdb/GeoLite2-City.mmdb").unwrap();
        let ips = vec![
            "75.63.106.74",
            "255.255.255.255",
            "gibberish",
            "132.206.246.203",
        ];

        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let ips = ips.iter().map(|i| i.to_object(py)).collect();
            let ips: PyReadonlyArray1<PyObject> = PyArray1::from_vec(py, ips).readonly();
            let res = geolocate(py, ips, &reader, vec![GeoColumn::City, GeoColumn::Latitude]);
            assert!(res.is_ok());
            let hm = res.unwrap();
            assert!(hm.contains_key(&GeoColumn::City));
            assert!(hm.contains_key(&GeoColumn::Latitude));

            let cities: Vec<Option<String>> = hm[&GeoColumn::City]
                .iter()
                .map(|c| c.extract(py).unwrap())
                .collect();
            assert_eq!(
                cities,
                vec![
                    Some("Houston".to_string()),
                    None,
                    None,
                    Some("Montreal".to_string())
                ]
            );

            let latitudes: Vec<Option<f64>> = hm[&GeoColumn::Latitude]
                .iter()
                .map(|c| c.extract(py).unwrap())
                .collect();
            assert_eq!(latitudes, vec![Some(29.9787), None, None, Some(45.5063)]);
        });
    }
}
