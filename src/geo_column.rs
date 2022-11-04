use std::str::FromStr;

use pyo3::{FromPyObject, IntoPy, PyAny, PyObject, PyResult, Python, ToPyObject};

use crate::errors::PandasMaxmindError;

/*
 Holds known column mappings

 todo: Derive FromPyObject https://pyo3.rs/v0.15.1/conversions/traits.html
*/
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum GeoColumn {
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
            _ => Err(PandasMaxmindError::ParseColumnError(s.to_string())),
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
