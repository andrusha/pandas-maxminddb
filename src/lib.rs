use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use maxminddb::{MaxMindDBError, Mmap, Reader};
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;
use pyo3::{pymodule, types::PyModule, PyObject, PyResult, Python};

use geo_column::GeoColumn;

use crate::errors::PandasMaxmindError;
use crate::lookup_result::LookupResult;
use crate::PandasMaxmindError::UnsupportedReaderError;

mod errors;
mod geo_column;
mod geolocate;
mod lookup_result;

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

fn result_into_py<'py, 'r>(
    py: Python<'py>,
    mut temp: HashMap<GeoColumn, Vec<LookupResult<'r>>>,
) -> HashMap<GeoColumn, &'py PyArray1<PyObject>> {
    let mut res = HashMap::with_capacity(temp.len());
    for (k, v) in temp.drain() {
        res.insert(
            k,
            v.into_iter()
                .map(|v| v.to_object(py))
                .collect::<Vec<PyObject>>()
                .into_pyarray(py),
        );
    }

    res
}

#[pyfunction]
fn mmdb_geolocate<'py>(
    py: Python<'py>,
    ips: PyReadonlyArray1<PyObject>,
    reader: PyObject,
    columns: Vec<GeoColumn>,
) -> PyResult<HashMap<GeoColumn, &'py PyArray1<PyObject>>> {
    let ips: Vec<String> = ips.as_array().iter().map(|i| i.to_string()).collect();

    match (
        reader.extract::<PyRef<PyReaderMem>>(py),
        reader.extract::<PyRef<PyReaderMmap>>(py),
    ) {
        (Ok(r), _) => {
            let reader = &r.reader;
            Ok(result_into_py(
                py,
                geolocate::geolocate(&ips, reader, columns)?,
            ))
        }
        (_, Ok(r)) => {
            let reader = &r.reader;
            Ok(result_into_py(
                py,
                geolocate::geolocate(&ips, reader, columns)?,
            ))
        }
        (_, _) => Err(UnsupportedReaderError)?,
    }
}

#[pymodule]
fn pandas_maxminddb(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyReader>()?;
    m.add_class::<PyReaderMem>()?;
    m.add_class::<PyReaderMmap>()?;
    m.add_function(wrap_pyfunction!(mmdb_geolocate, m)?)?;

    Ok(())
}
