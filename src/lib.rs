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

// country=none_if_exception(lambda: geo["country"]["names"]["en"]),
// state=none_if_exception(lambda: geo["subdivisions"][0]["names"]["en"]),
// city=none_if_exception(lambda: geo["city"]["names"]["en"]),
// postcode=none_if_exception(lambda: geo["postal"]["code"]),
// longitude=none_if_exception(lambda: float(geo["location"]["longitude"])),
// latitude=none_if_exception(lambda: float(geo["location"]["latitude"])),
// accuracy_radius=none_if_exception(lambda: int(geo["location"]["accuracy_radius"])),

#[pyfunction]
fn mmdb_geolocate<'py>(
  py: Python<'py>,
  ips: PyReadonlyArray1<PyObject>,
  mmdb_path: &str,
  columns: Vec<GeoColumn>
) -> HashMap<GeoColumn, &'py PyArray1<PyObject>> {
  let reader = maxminddb::Reader::open_readfile(mmdb_path).unwrap();

  // let res: HashMap<GeoColumn, Vec<PyObject>> = HashMap::new();
  // for c in columns.into_iter() {
  //   res[c.into()] = ips.len();
  // }

  let cities = ips
      .as_array()
      .map(|s| {
        let ip = s.to_string().parse::<IpAddr>().unwrap();
        let city: geoip2::City = reader.lookup(ip).unwrap();
        let ccode = city.country.map(|c| c.iso_code).flatten().unwrap_or("NA");

        PyString::new(py, ccode).to_object(py)
      })
      .into_pyarray(py);

  let mut res = HashMap::new();
  res.insert(GeoColumn::City, cities);

  res
}

#[pymodule]
fn pandas_maxminddb(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
  m.add_function(wrap_pyfunction!(mmdb_geolocate, m)?)?;

  Ok(())
}
