use std::collections::HashMap;

use maxminddb::{MaxMindDBError, Mmap, Reader};
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;
use pyo3::{pymodule, types::PyModule, PyObject, PyResult, Python};

use geo_column::GeoColumn;

use crate::errors::PandasMaxmindError;
use crate::lookup_result::LookupResult;
use crate::PandasMaxmindError::{ParallelMmapReaderError, UnsupportedReaderError};

mod errors;
mod geo_column;
mod geolocate;
mod lookup_result;

/*
   Generic superclass, useful for Python code
*/
#[pyclass(subclass, name = "Reader")]
struct PyReader;

/*
   Loads DB fully into memory
*/
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

/*
   Uses memory mapping instead of loading whole thing into memory
*/
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

/*
   Converts Vec of lookup results into NDArray,
   needs reallocation to spawn all the python objects
*/
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

/*
   Interface exposed to internal library Python code
*/
#[pyfunction]
fn mmdb_geolocate<'py>(
    py: Python<'py>,
    ips: PyReadonlyArray1<PyObject>,
    reader: PyObject,
    columns: Vec<GeoColumn>,
    parallel: bool,
    parallel_chunk_size: usize,
) -> PyResult<HashMap<GeoColumn, &'py PyArray1<PyObject>>> {
    let ips: Vec<String> = ips.as_array().iter().map(|i| i.to_string()).collect();

    // Has to do the match, since PyO3 doesn't allow generic functions
    match (
        reader.extract::<PyRef<PyReaderMem>>(py),
        reader.extract::<PyRef<PyReaderMmap>>(py),
    ) {
        (Ok(r), _) => {
            let reader = &r.reader;
            let temp = if parallel {
                geolocate::geolocate_par(&ips, reader, &columns, parallel_chunk_size)?
            } else {
                geolocate::geolocate(&ips, reader, &columns)?
            };

            Ok(result_into_py(py, temp))
        }
        (_, Ok(r)) => {
            let reader = &r.reader;
            if parallel {
                Err(ParallelMmapReaderError)?
            } else {
                Ok(result_into_py(
                    py,
                    geolocate::geolocate(&ips, reader, &columns)?,
                ))
            }
        }
        // Might as well be incorrect object being passed like None
        (_, _) => Err(UnsupportedReaderError(
            reader
                .as_ref(py)
                .get_type()
                .name()
                .unwrap_or("Unknown")
                .to_string(),
        ))?,
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
