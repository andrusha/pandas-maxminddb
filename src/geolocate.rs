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
