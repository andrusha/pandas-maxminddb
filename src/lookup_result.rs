use core::option::Option;
use pyo3::{IntoPy, PyObject, Python, ToPyObject};

pub enum LookupResult<'py> {
    String(Option<&'py str>),
    Float(Option<f64>),
    Int(Option<u16>),
}

impl<'py> Into<LookupResult<'py>> for Option<&'py str> {
    fn into(self) -> LookupResult<'py> {
        LookupResult::String(self)
    }
}

impl<'py> Into<LookupResult<'py>> for Option<f64> {
    fn into(self) -> LookupResult<'py> {
        LookupResult::Float(self)
    }
}

impl<'py> Into<LookupResult<'py>> for Option<u16> {
    fn into(self) -> LookupResult<'py> {
        LookupResult::Int(self)
    }
}

impl<'py> ToPyObject for LookupResult<'py> {
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
