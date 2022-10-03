use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;

use maxminddb::geoip2;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1, PyReadwriteArray1};
use pyo3::{pymodule, PyObject, PyResult, Python, ToPyObject, types::PyModule};
use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;
use pyo3::types::PyString;
use thiserror::Error;

// country=none_if_exception(lambda: geo["country"]["names"]["en"]),
// state=none_if_exception(lambda: geo["subdivisions"][0]["names"]["en"]),
// city=none_if_exception(lambda: geo["city"]["names"]["en"]),
// postcode=none_if_exception(lambda: geo["postal"]["code"]),
// longitude=none_if_exception(lambda: float(geo["location"]["longitude"])),
// latitude=none_if_exception(lambda: float(geo["location"]["latitude"])),
// accuracy_radius=none_if_exception(lambda: int(geo["location"]["accuracy_radius"])),

// todo: into PyErr
#[derive(Error, Debug)]
enum PandasMaxmindError {
  #[error("invalid geo column name: {0}")]
  ParseColumnError(String)
}

impl From<PandasMaxmindError> for PyErr {
  fn from(e: PandasMaxmindError) -> Self {
    match e {
      PandasMaxmindError::ParseColumnError(_) => PyKeyError::new_err(e.to_string())
    }
  }
}

// Derive FromPyObject https://pyo3.rs/v0.15.1/conversions/traits.html
#[derive(Debug, Hash, Eq, PartialEq)]
enum GeoColumn {
  Country,
  State,
  City,
  Postcode,
  Longitude,
  Latitude,
  AccuracyRadius,
}

impl FromStr for GeoColumn {
  type Err = PandasMaxmindError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    use GeoColumn::*;

    match s {
      "country" => Ok(Country),
      "state" => Ok(State),
      "city" => Ok(City),
      "postcode" => Ok(Postcode),
      "longitude" => Ok(Longitude),
      "latitude" => Ok(Latitude),
      "accuracy_radius" => Ok(AccuracyRadius),
      _ => Err(PandasMaxmindError::ParseColumnError(s.to_string()))
    }
  }
}

impl ToPyObject for GeoColumn {
  fn to_object(&self, py: Python<'_>) -> PyObject {
    match self {
      GeoColumn::Country => "country".into_py(py),
      GeoColumn::State => "state".into_py(py),
      GeoColumn::City => "city".into_py(py),
      GeoColumn::Postcode => "postcode".into_py(py),
      GeoColumn::Longitude => "longitude".into_py(py),
      GeoColumn::Latitude => "latitude".into_py(py),
      GeoColumn::AccuracyRadius => "accuracy_radius".into_py(py),
    }
  }
}

impl IntoPy<PyObject> for GeoColumn {
  fn into_py(self, py: Python<'_>) -> PyObject {
    self.to_object(py)
  }
}

impl<'source> FromPyObject<'source> for GeoColumn {
  fn extract(ob: &'source PyAny) -> PyResult<Self> {
    Ok(GeoColumn::from_str(ob.extract()?)?)
  }
}

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
