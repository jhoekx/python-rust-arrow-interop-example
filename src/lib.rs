//    Copyright 2021 Apache Software Foundation (ASF)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Slightly adapted from
// https://github.com/apache/arrow/commit/1d2b4a55770fa4dbe24959b3b40c745964c3184e

use std::{error, fmt, sync::Arc};

use arrow::{array::{ArrayRef, Int64Array, make_array_from_raw}, compute::kernels, error::ArrowError, ffi};
use pyo3::{exceptions::PyOSError, prelude::*};
use libc::uintptr_t;
use pyo3::wrap_pyfunction;

#[derive(Debug)]
enum PyO3ArrowError {
    ArrowError(ArrowError),
}

impl fmt::Display for PyO3ArrowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PyO3ArrowError::ArrowError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for PyO3ArrowError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            PyO3ArrowError::ArrowError(ref e) => Some(e),
        }
    }
}

impl From<ArrowError> for PyO3ArrowError {
    fn from(err: ArrowError) -> PyO3ArrowError {
        PyO3ArrowError::ArrowError(err)
    }
}

impl From<PyO3ArrowError> for PyErr {
    fn from(err: PyO3ArrowError) -> PyErr {
        PyOSError::new_err(err.to_string())
    }
}

fn to_rust(ob: PyObject, py: Python) -> PyResult<ArrayRef> {
    // prepare a pointer to receive the Array struct
    let (array_pointer, schema_pointer) =
        ffi::ArrowArray::into_raw(unsafe { ffi::ArrowArray::empty() });

    // make the conversion through PyArrow's private API
    // this changes the pointer's memory and is thus unsafe. In particular, `_export_to_c` can go out of bounds
    ob.call_method1(
        py,
        "_export_to_c",
        (array_pointer as uintptr_t, schema_pointer as uintptr_t),
    )?;

    let array = unsafe { make_array_from_raw(array_pointer, schema_pointer) }
        .map_err(PyO3ArrowError::from)?;
    Ok(array)
}

fn to_py(array: ArrayRef, py: Python) -> PyResult<PyObject> {
    let (array_pointer, schema_pointer) =
        array.to_raw().map_err(PyO3ArrowError::from)?;

    let pa = py.import("pyarrow")?;

    let array = pa.getattr("Array")?.call_method1(
        "_import_from_c",
        (array_pointer as uintptr_t, schema_pointer as uintptr_t),
    )?;
    Ok(array.to_object(py))
}

#[pyfunction]
fn double(array: PyObject, py: Python) -> PyResult<PyObject> {
    // import
    let array = to_rust(array, py)?;

    // perform some operation
    let array =
        array
            .as_any()
            .downcast_ref::<Int64Array>()
            .ok_or_else(|| PyO3ArrowError::ArrowError(ArrowError::ParseError(
                "Expects an int64".to_string(),
            )))?;
    let array =
        kernels::arithmetic::add(&array, &array).map_err(PyO3ArrowError::from)?;
    let array = Arc::new(array);

    // export
    to_py(array, py)
}

/// A Python module implemented in Rust.
#[pymodule]
fn arrow_example(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(double, m)?)?;

    Ok(())
}
