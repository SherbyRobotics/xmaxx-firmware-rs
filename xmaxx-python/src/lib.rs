use pyo3::prelude::*;
use pyo3::exceptions::PyException;

use serialport;
use serialport::SerialPort;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A command to be sent to the firmware.
#[pyclass]
struct Command;

/// A socket to communicate with the Xmaxx firmware.
#[pyclass]
struct XmaxxFirmware {
    port: Option<Box<dyn SerialPort>>
}

#[pymethods]
impl XmaxxFirmware {
    #[new]
    #[pyo3(text_signature = "(port: str, baudrate: int)")]
    fn new(port: &str, baudrate: u32) -> PyResult<Self> {
        match serialport::new(port, baudrate).open() {
            Ok(port) => Ok(Self { port: Some(port) }),
            Err(err) => Err(PyException::new_err(err.description))
        }
    }

    #[pyo3(text_signature = "(command: Command)")]
    fn send(&mut self, command: &Command) {
        todo!()
    }

    /// Close the connection to the firmware.
    ///
    /// The port can be reopened after this method call.
    fn close(&mut self) {
        // cannot move self to drop
        // therefore Self must manage its internal state in a non-Rustic fashion
        self.port = None;
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn xmaxx_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<XmaxxFirmware>()?;
    m.add_class::<Command>()?;
    Ok(())
}
