use std::collections::HashMap;
use std::net::IpAddr;

use maxminddb::geoip2;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1, PyReadwriteArray1};
use pyo3::{pymodule, PyObject, PyResult, Python, ToPyObject, types::PyModule};
use pyo3::prelude::*;
use pyo3::types::PyString;
use geo_column::GeoColumn;

mod geo_column;
mod errors;

#[pyfunction]
fn mmdb_geolocate<'py>(
  py: Python<'py>,
  ips: PyReadonlyArray1<PyObject>,
  mmdb_path: &str,
  columns: Vec<GeoColumn>,
) -> HashMap<GeoColumn, &'py PyArray1<PyObject>> {
  let reader = maxminddb::Reader::open_readfile(mmdb_path).unwrap();

  let mut temp: HashMap<GeoColumn, Vec<PyObject>> = HashMap::with_capacity(columns.len());
  for &c in columns.iter() {
    temp.insert(c, Vec::with_capacity(ips.len()));
  }

  for (i, ip) in ips.as_array().iter().enumerate() {
    let ip = ip.to_string().parse::<IpAddr>().unwrap();
    let lookup: geoip2::City = reader.lookup(ip).unwrap();

    for c in columns.iter() {
      match c {
        // country=none_if_exception(lambda: geo["country"]["names"]["en"]),
        GeoColumn::Country => {
          let v = lookup.country.as_ref()
              .map(|c| c.iso_code).flatten()
              .unwrap_or("NA").to_object(py);
          temp.get_mut(c).unwrap().push(v);
        }
        // state=none_if_exception(lambda: geo["subdivisions"][0]["names"]["en"]),
        GeoColumn::State => {
          let v = lookup.subdivisions.as_ref()
              .map(|sd| {
                sd.first().map(|s| s.iso_code).flatten()
              }).flatten()
              .unwrap_or("NA").to_object(py);
          temp.get_mut(c).unwrap().push(v);
        }
        // city=none_if_exception(lambda: geo["city"]["names"]["en"]),
        GeoColumn::City => {
          // lookup.country.map(|c| c.iso_code).flatten().unwrap_or("NA").to_object(py);
        }
        // postcode=none_if_exception(lambda: geo["postal"]["code"]),
        GeoColumn::Postcode => {
          let v = lookup.postal.as_ref()
              .map(|c| c.code).flatten()
              .unwrap_or("NA").to_object(py);
          temp.get_mut(c).unwrap().push(v);
        }
        // longitude=none_if_exception(lambda: float(geo["location"]["longitude"])),
        GeoColumn::Longitude => {
          let v = lookup.location.as_ref()
              .map(|l| l.longitude).flatten()
              .map(|l| l.to_string())
              .unwrap_or("NA".to_string()).to_object(py);
          temp.get_mut(c).unwrap().push(v);
        }
        // latitude=none_if_exception(lambda: float(geo["location"]["latitude"])),
        GeoColumn::Latitude => {
          let v = lookup.location.as_ref()
              .map(|l| l.latitude).flatten()
              .map(|l| l.to_string())
              .unwrap_or("NA".to_string()).to_object(py);
          temp.get_mut(c).unwrap().push(v);
        }
        // accuracy_radius=none_if_exception(lambda: int(geo["location"]["accuracy_radius"])),
        GeoColumn::AccuracyRadius => {
          // lookup.location
          //     .map(|l| l.accuracy_radius).flatten()
          //     .map(|l| l.to_string())
          //     .unwrap_or("NA".to_string()).to_object(py);
        }
      }
    }
  }

  // Convert to the PyArray
  let mut res = HashMap::with_capacity(temp.len());
  for (k, v) in temp {
    res.insert(k, PyArray1::from_vec(py, v));
  }

  res
}

#[pymodule]
fn pandas_maxminddb(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
  m.add_function(wrap_pyfunction!(mmdb_geolocate, m)?)?;

  Ok(())
}
