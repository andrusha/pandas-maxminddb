use std::collections::HashMap;
use std::net::IpAddr;

use maxminddb::{geoip2, MaxMindDBError, Mmap, Reader};
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;
use pyo3::{pymodule, types::PyModule, PyObject, PyResult, Python, ToPyObject};

use geo_column::GeoColumn;

use crate::errors::PandasMaxmindError;
use crate::PandasMaxmindError::UnsupportedReaderError;

mod errors;
mod geo_column;

#[pyclass(subclass, name = "Reader")]
struct PyReader;

// DB is loaded into memory fully, this is container class
// which is later used to create a context
#[pyclass(name = "ReaderMem", extends = PyReader)]
struct PyReaderMem {
    reader: Reader<Vec<u8>>,
}

#[pymethods]
impl PyReaderMem {
    #[new]
    fn new(mmdb_path: &str) -> PyResult<(Self, PyReader)> {
        let reader = Reader::open_readfile(mmdb_path)
            .map_err(<MaxMindDBError as Into<PandasMaxmindError>>::into)?;

        Ok((PyReaderMem { reader }, PyReader))
    }
}

// Memory mapping version of reader
#[pyclass(name = "ReaderMmap", extends = PyReader)]
struct PyReaderMmap {
    reader: Reader<Mmap>,
}

#[pymethods]
impl PyReaderMmap {
    #[new]
    fn new(mmdb_path: &str) -> PyResult<(Self, PyReader)> {
        let reader = Reader::open_mmap(mmdb_path)
            .map_err(<MaxMindDBError as Into<PandasMaxmindError>>::into)?;

        Ok((PyReaderMmap { reader }, PyReader))
    }
}

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

fn geolocate<'py, T: AsRef<[u8]>>(
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

#[pyfunction]
fn mmdb_geolocate<'py>(
    py: Python<'py>,
    ips: PyReadonlyArray1<PyObject>,
    reader: PyObject,
    columns: Vec<GeoColumn>,
) -> PyResult<HashMap<GeoColumn, &'py PyArray1<PyObject>>> {
    let mut temp = match (
        reader.extract::<PyRef<PyReaderMem>>(py),
        reader.extract::<PyRef<PyReaderMmap>>(py),
    ) {
        (Ok(r), _) => geolocate(py, ips, &r.reader, columns),
        (_, Ok(r)) => geolocate(py, ips, &r.reader, columns),
        (_, _) => Err(UnsupportedReaderError),
    }?;

    let mut res = HashMap::with_capacity(temp.len());
    for (k, v) in temp.drain() {
        res.insert(k, v.into_pyarray(py));
    }

    Ok(res)
}

#[pymodule]
fn pandas_maxminddb(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyReader>()?;
    m.add_class::<PyReaderMem>()?;
    m.add_class::<PyReaderMmap>()?;
    m.add_function(wrap_pyfunction!(mmdb_geolocate, m)?)?;

    Ok(())
}
