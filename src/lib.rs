use std::net::IpAddr;
use numpy::{IntoPyArray, PyArrayDyn, PyReadonlyArrayDyn};
use numpy::ndarray::{ArrayD, ArrayViewD, ArrayViewMutD};
use pyo3::{pymodule, PyObject, PyResult, Python, ToPyObject, types::PyModule};
use pyo3::types::PyString;
use maxminddb::geoip2;

#[pymodule]
fn pandas_maxminddb(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
  #[pyfn(m)]
  #[pyo3()]
  fn mmdb_geolocate<'py>(
    py: Python<'py>,
    ips: PyReadonlyArrayDyn<PyObject>,
  ) -> &'py PyArrayDyn<PyObject> {
    let reader = maxminddb::Reader::open_readfile("GeoLite2-City.mmdb").unwrap();

    ips
        .as_array()
        .map(|s| {
          let ip = s.to_string().parse::<IpAddr>().unwrap();
          let city: geoip2::City = reader.lookup(ip).unwrap();
          let ccode = city.country.map(|c| c.iso_code).flatten().unwrap_or("NA");

          PyString::new(py, ccode).to_object(py)
        })
        .into_pyarray(py)
  }

  Ok(())
}
