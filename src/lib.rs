use std::collections::HashMap;
use std::net::IpAddr;

use maxminddb::{geoip2, Reader};
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;
use pyo3::{pymodule, types::PyModule, PyObject, PyResult, Python, ToPyObject};

use geo_column::GeoColumn;

mod errors;
mod geo_column;

fn geolocate<'py>(
    py: Python<'py>,
    ips: PyReadonlyArray1<PyObject>,
    reader: &Reader<Vec<u8>>,
    columns: Vec<GeoColumn>,
) -> HashMap<GeoColumn, Vec<PyObject>> {
    let mut res = HashMap::with_capacity(columns.len());
    for &c in columns.iter() {
        res.insert(c, Vec::with_capacity(ips.len()));
    }

    for ip in ips.as_array().iter() {
        let ip = ip.to_string().parse::<IpAddr>().unwrap();
        let lookup: geoip2::City = reader.lookup(ip).unwrap();

        for (col, vec) in res.iter_mut() {
            let v = match col {
                GeoColumn::Country => lookup
                    .country
                    .as_ref()
                    .and_then(|c| c.iso_code)
                    .to_object(py),

                GeoColumn::State => lookup
                    .subdivisions
                    .as_ref()
                    .and_then(|sd| sd.first())
                    .and_then(|s| s.iso_code)
                    .to_object(py),

                GeoColumn::City => lookup
                    .city
                    .as_ref()
                    .and_then(|c| c.names.as_ref())
                    .and_then(|n| n.get("en"))
                    .to_object(py),

                GeoColumn::Postcode => lookup.postal.as_ref().and_then(|c| c.code).to_object(py),

                GeoColumn::Longitude => lookup
                    .location
                    .as_ref()
                    .and_then(|l| l.longitude)
                    .to_object(py),

                GeoColumn::Latitude => lookup
                    .location
                    .as_ref()
                    .and_then(|l| l.latitude)
                    .to_object(py),

                GeoColumn::AccuracyRadius => lookup
                    .location
                    .as_ref()
                    .and_then(|l| l.accuracy_radius)
                    .to_object(py),
            };

            vec.push(v);
        }
    }

    res
}

#[pyfunction]
fn mmdb_geolocate<'py>(
    py: Python<'py>,
    ips: PyReadonlyArray1<PyObject>,
    mmdb_path: &str,
    columns: Vec<GeoColumn>,
) -> HashMap<GeoColumn, &'py PyArray1<PyObject>> {
    let reader = maxminddb::Reader::open_readfile(mmdb_path).unwrap();

    let mut temp = geolocate(py, ips, &reader, columns);
    let mut res = HashMap::with_capacity(temp.len());
    for (k, v) in temp.drain() {
        res.insert(k, v.into_pyarray(py));
    }

    res
}

#[pymodule]
fn pandas_maxminddb(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(mmdb_geolocate, m)?)?;

    Ok(())
}
