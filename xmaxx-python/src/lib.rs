use std::time::Duration;

use pyo3::exceptions::PyException;
use pyo3::prelude::*;

use serialport;
use serialport::SerialPort;

use xmaxx_messages::*;

/// A command to be sent to the firmware.
#[pyclass(name = "Command")]
struct PyCommand {
    steering: f32,
    fl_whl_rpm: f32,
    fr_whl_rpm: f32,
    rl_whl_rpm: f32,
    rr_whl_rpm: f32,
}

#[pymethods]
impl PyCommand {
    #[new]
    #[pyo3(text_signature = "(steering, fl_whl_rpm, fr_whl_rpm, rl_whl_rpm, rr_whl_rpm)")]
    fn new(
        steering: f32,
        fl_whl_rpm: f32,
        fr_whl_rpm: f32,
        rl_whl_rpm: f32,
        rr_whl_rpm: f32,
    ) -> Self {
        Self {
            steering,
            fl_whl_rpm,
            fr_whl_rpm,
            rl_whl_rpm,
            rr_whl_rpm,
        }
    }
}

impl Into<Command> for &PyCommand {
    fn into(self) -> Command {
        Command::new(
            self.steering,
            self.fl_whl_rpm,
            self.fr_whl_rpm,
            self.rl_whl_rpm,
            self.rr_whl_rpm,
        )
    }
}

/// Wrapper type around [`XmaxxEvent`].
///
/// It is not a Python object but it converts to two: [`PySensors`] and
/// [`PyXmaxxInfo`]. A Python function returning this types can be be annotated
/// with `Union[Sensors, XmaxxInfo]`.
enum PyXmaxxEvent {
    Sensors(PySensors),
    Info(PyXmaxxInfo),
}

impl IntoPy<PyObject> for PyXmaxxEvent {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            Self::Sensors(sensors) => sensors.into_py(py),
            Self::Info(info) => info.into_py(py),
        }
    }
}

/// Sensor information from the firmware.
#[pyclass(name = "Sensors")]
struct PySensors {
    /// Front left wheel RPM.
    #[pyo3(get)]
    fl_whl_rpm: f32,
    /// Front right wheel RPM.
    #[pyo3(get)]
    fr_whl_rpm: f32,
    /// Rear left wheel RPM.
    #[pyo3(get)]
    rl_whl_rpm: f32,
    /// Rear right wheel RPM.
    #[pyo3(get)]
    rr_whl_rpm: f32,
}

#[pymethods]
impl PySensors {
    fn __repr__(&self) -> String {
        format!(
            "Sensors(fl_whl_rpm={}, fr_whl_rpm={}, rl_whl_rpm={}, rr_whl_rpm={})",
            self.fl_whl_rpm, self.fr_whl_rpm, self.rl_whl_rpm, self.rr_whl_rpm
        )
    }
}

impl From<Sensors> for PySensors {
    fn from(sensors: Sensors) -> Self {
        Self {
            fl_whl_rpm: sensors.fl_whl_rpm,
            fr_whl_rpm: sensors.fr_whl_rpm,
            rl_whl_rpm: sensors.rl_whl_rpm,
            rr_whl_rpm: sensors.rr_whl_rpm,
        }
    }
}

/// Information about what is happening in the firmware.
#[pyclass(name = "XmaxxInfo")]
enum PyXmaxxInfo {
    /// The firmware could not serialize a message.
    SerializationError,
    /// The firmware could not deserialize a message.
    DeserializationError,
    /// The software read buffer overflowed.
    ReadBufferOverflow,
    /// It was too long since the last message received.
    ReadTimeout,
}

impl From<XmaxxInfo> for PyXmaxxInfo {
    fn from(info: XmaxxInfo) -> Self {
        match info {
            XmaxxInfo::SerializationError => Self::SerializationError,
            XmaxxInfo::DeserializationError => Self::DeserializationError,
            XmaxxInfo::ReadBufferOverflow => Self::ReadBufferOverflow,
            XmaxxInfo::ReadTimeout => Self::ReadTimeout,
        }
    }
}

/// A socket to communicate with the Xmaxx firmware.
#[pyclass]
struct XmaxxFirmware {
    port: Option<Box<dyn SerialPort>>,
}

#[pymethods]
impl XmaxxFirmware {
    #[new]
    #[pyo3(signature = (port, baudrate=57600, timeout=500))]
    fn new(port: &str, baudrate: u32, timeout: u64) -> PyResult<Self> {
        match serialport::new(port, baudrate).open() {
            Ok(mut port) => {
                // must set timeout otherwise it is 0 and every operation hits it
                port.set_timeout(Duration::from_millis(timeout))
                    .expect("setting timeout should just work?");
                Ok(Self { port: Some(port) })
            }
            Err(err) => Err(PyException::new_err(err.description)),
        }
    }

    /// Sends a command to the firmware.
    ///
    /// Raises an exception if the socket was closed or if an error occurs
    /// during the write operation.
    fn send(&mut self, command: &PyCommand) -> PyResult<()> {
        if let Some(port) = &mut self.port {
            let mut buf = [0u8; 128]; // this buffer could be smaller I think
            let msg = serialize::<Command>(&command.into(), &mut buf)
                .expect("serializing should just work");

            for i in 0..msg.len() {
                port.write(&msg[i..=i])?;
            }

            Ok(())
        } else {
            Err(PyException::new_err("the socket was closed"))
        }
    }

    /// Reads information from the firmware.
    ///
    /// Raises errors on failed io operations and if it fails to deserialize
    /// a message.
    ///
    /// This method returns either a `Sensors` or a `XmaxxInfo`. Therefore,
    /// it is recommended to match its output a little like this:
    /// ```python
    /// from xmaxx_python import XmaxxFirmware, Sensors, XmaxxInfo
    ///
    /// firmware = XmaxxFirmware("/path/to/port")
    ///
    /// match firmware.read():
    ///     case Sensors() as sensors:
    ///         ...
    ///     case XmaxxInfo() as info:
    ///         ...
    /// ```
    fn read(&mut self) -> PyResult<PyXmaxxEvent> {
        if let Some(port) = &mut self.port {
            let mut b = [0u8; 1];
            let mut buf = Vec::<u8>::new();

            loop {
                port.read(&mut b)?;
                buf.push(b[0]);

                if buf.last() == Some(&0u8) {
                    break;
                }
            }

            let info = deserialize(buf.as_mut_slice())
                .or_else(|_| Err(PyException::new_err("could not deserialize")))?;

            Ok(match info {
                XmaxxEvent::Sensors(sensors) => PyXmaxxEvent::Sensors(sensors.into()),
                XmaxxEvent::Info(info) => PyXmaxxEvent::Info(info.into()),
            })
        } else {
            Err(PyException::new_err("the socket was closed"))
        }
    }

    /// Close the connection to the firmware.
    ///
    /// The calling instance can no longer be used after. To reopen the
    /// communication, instantiate a new object.
    fn close(&mut self) {
        // cannot move self to drop
        // setting `self.port` to none drops the serial port therefore closing it
        self.port = None;
    }
}

/// A Python module to interface with the Xmaxx firmware - in Rust.
#[pymodule]
fn xmaxx_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<XmaxxFirmware>()?;
    m.add_class::<PyCommand>()?;
    m.add_class::<PySensors>()?;
    m.add_class::<PyXmaxxInfo>()?;
    Ok(())
}
