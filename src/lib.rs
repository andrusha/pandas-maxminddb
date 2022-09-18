use numpy::{IntoPyArray, PyArrayDyn, PyReadonlyArrayDyn};
use numpy::ndarray::{ArrayD, ArrayViewD, ArrayViewMutD};
use pyo3::{pymodule, PyObject, PyResult, Python, ToPyObject, types::PyModule};
use pyo3::types::PyString;

#[pymodule]
fn numpy_maxminddb(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
  // example using immutable borrows producing a new array
  fn axpy(a: f64, x: ArrayViewD<'_, f64>, y: ArrayViewD<'_, f64>) -> ArrayD<f64> {
    a * &x + &y
  }

  // example using a mutable borrow to modify an array in-place
  fn mult(a: f64, mut x: ArrayViewMutD<'_, f64>) {
    x *= a;
  }

  // wrapper of `axpy`
  #[pyfn(m)]
  #[pyo3(name = "axpy")]
  fn axpy_py<'py>(
    py: Python<'py>,
    a: f64,
    x: PyReadonlyArrayDyn<f64>,
    y: PyReadonlyArrayDyn<f64>,
  ) -> &'py PyArrayDyn<f64> {
    let x = x.as_array();
    let y = y.as_array();
    let z = axpy(a, x, y);
    z.into_pyarray(py)
  }

  // wrapper of `axpy`
  #[pyfn(m)]
  #[pyo3()]
  fn hello_world<'py>(
    py: Python<'py>,
    x: PyReadonlyArrayDyn<PyObject>,
  ) -> &'py PyArrayDyn<PyObject> {
    let x = x.as_array();
    x.map(|s| format!("Hello {}!", s.to_string())).map(|s| PyString::new(py, s).to_object(py)).into_pyarray(py)
  }

  // wrapper of `mult`
  #[pyfn(m)]
  #[pyo3(name = "mult")]
  fn mult_py(_py: Python<'_>, a: f64, x: &PyArrayDyn<f64>) {
    let x = unsafe { x.as_array_mut() };
    mult(a, x);
  }

  Ok(())
}
