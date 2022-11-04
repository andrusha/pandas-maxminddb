use core::option::Option;
use pyo3::{IntoPy, PyObject, Python, ToPyObject};

/*
   Holds lookup result until it's converted to Python objects
   In order to enable parallelism as allocating Python objects is subject to GIL
   and is not thread safe
*/
pub enum LookupResult<'a> {
    String(Option<&'a str>),
    Float(Option<f64>),
    Int(Option<u16>),
}

impl<'a> From<Option<&'a str>> for LookupResult<'a> {
    fn from(s: Option<&'a str>) -> Self {
        LookupResult::String(s)
    }
}

impl<'a> From<Option<f64>> for LookupResult<'a> {
    fn from(v: Option<f64>) -> Self {
        LookupResult::Float(v)
    }
}

impl<'a> From<Option<u16>> for LookupResult<'a> {
    fn from(v: Option<u16>) -> Self {
        LookupResult::Int(v)
    }
}

impl<'a> ToPyObject for LookupResult<'a> {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            LookupResult::String(s) => s.into_py(py),
            LookupResult::Float(f) => f.into_py(py),
            LookupResult::Int(i) => i.into_py(py),
        }
    }
}

impl<'py> IntoPy<PyObject> for LookupResult<'py> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}
