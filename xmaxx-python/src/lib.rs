#![doc = include_str!("../README.md")]

use std::thread;
use std::time::Duration;

use pyo3::exceptions::PyException;
use pyo3::prelude::*;

use serialport;
use serialport::SerialPort;

use xmaxx_messages::*;

/// A command to be sent to the firmware.
#[pyclass(name = "Command")]
struct PyCommand {
    steering: i32,
    fl_whl_rpm: i32,
    fr_whl_rpm: i32,
    rl_whl_rpm: i32,
    rr_whl_rpm: i32,
}

#[pymethods]
impl PyCommand {
    #[new]
    fn new(
        steering: i32,
        fl_whl_rpm: i32,
        fr_whl_rpm: i32,
        rl_whl_rpm: i32,
        rr_whl_rpm: i32,
    ) -> Self {
        Self {
            steering,
            fl_whl_rpm,
            fr_whl_rpm,
            rl_whl_rpm,
            rr_whl_rpm,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Command(steering={}, fl_whl_rpm={}, fr_whl_rpm={}, rl_whl_rpm={}, rr_whl_rpm={})",
            self.steering, self.fl_whl_rpm, self.fr_whl_rpm, self.rl_whl_rpm, self.rr_whl_rpm
        )
    }
}

impl Into<Command> for &PyCommand {
    fn into(self) -> Command {
        Command {
            steering: self.steering,
            fl_whl_rpm: self.fl_whl_rpm,
            fr_whl_rpm: self.fr_whl_rpm,
            rl_whl_rpm: self.rl_whl_rpm,
            rr_whl_rpm: self.rr_whl_rpm,
        }
    }
}

/// Wrapper type around [`Info`].
///
/// It is not a Python object but it converts to two: [`PySensors`] and
/// [`PyLog`]. A Python function returning this types can be annotated
/// with `Union[Sensors, Log]`.
enum PyInfo {
    Sensors(PySensors),
    Log(PyLog),
}

impl IntoPy<PyObject> for PyInfo {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            Self::Sensors(sensors) => sensors.into_py(py),
            Self::Log(log) => log.into_py(py),
        }
    }
}

/// Sensor information from the firmware.
#[pyclass(name = "Sensors")]
struct PySensors {
    /// Front left wheel RPM.
    #[pyo3(get)]
    fl_whl_rpm: i32,
    /// Front right wheel RPM.
    #[pyo3(get)]
    fr_whl_rpm: i32,
    /// Rear left wheel RPM.
    #[pyo3(get)]
    rl_whl_rpm: i32,
    /// Rear right wheel RPM.
    #[pyo3(get)]
    rr_whl_rpm: i32,
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
#[pyclass(name = "Log")]
enum PyLog {
    /// The firmware could not serialize a message.
    SerializationError,
    /// The firmware could not deserialize a message.
    DeserializationError,
    /// The software read buffer overflowed.
    ReadBufferOverflow,
    /// It was too long since the last message received.
    ReadTimeout,
    /// The firmware panicked and must must reseted.
    FirmwarePanic,
    /// The command sent was invalid.
    InvalidCommand,
    /// A command was received.
    CommandReceived,
    /// No command was received.
    NoCommandReceived,
}

impl From<Log> for PyLog {
    fn from(log: Log) -> Self {
        match log {
            Log::SerializationError => Self::SerializationError,
            Log::DeserializationError => Self::DeserializationError,
            Log::ReadBufferOverflow => Self::ReadBufferOverflow,
            Log::ReadTimeout => Self::ReadTimeout,
            Log::FirmwarePanic => Self::FirmwarePanic,
            Log::InvalidCommand => Self::InvalidCommand,
            Log::CommandReceived => Self::CommandReceived,
            Log::NoCommandReceived => Self::NoCommandReceived,
        }
    }
}

/// A socket to communicate with the Xmaxx firmware.
///
/// **Note:** if there are problems with deserialization in the firmware,
/// it might be because the computer is sending the next bytes too soon.
/// Try increasing the send delay.
#[pyclass(name = "Firmware")]
struct PyFirmware {
    port: Option<Box<dyn SerialPort>>,
    send_delay: Duration,
}

#[pymethods]
impl PyFirmware {
    /// Instantiates a connection to the firmware.
    ///
    /// Parameters:
    /// -----------
    /// port: str
    ///     the path to the serial port
    /// baudrate: int = 57600
    ///     the baudrate of the communication
    /// timeout: int = 500
    ///     the timeout on io operations (ms)
    /// send_delay: int = 3
    ///     the delay between each byte sent (ms)
    #[new]
    #[pyo3(signature = (port, baudrate=57600, timeout=500, send_delay=3))]
    fn new(port: &str, baudrate: u32, timeout: u64, send_delay: u64) -> PyResult<Self> {
        match serialport::new(port, baudrate).open() {
            Ok(mut port) => {
                // must set timeout otherwise it is 0 and every operation hits it
                port.set_timeout(Duration::from_millis(timeout))
                    .expect("setting timeout should just work?");
                Ok(Self {
                    port: Some(port),
                    send_delay: Duration::from_millis(send_delay),
                })
            }
            Err(err) => Err(PyException::new_err(err.description)),
        }
    }

    /// Sends a command to the firmware.
    ///
    /// This function sleeps `send_delay` for each byte sent. Treat like a
    /// blocking function.
    ///
    /// Raises an exception if the socket was closed or if an io error occurs
    /// during the write operation.
    ///
    /// BUG After a while without sending, operation times out systematically
    /// until a new firmware instantiated.
    ///
    /// Parameters:
    /// -----------
    /// command: Command
    ///     the command to send to the firmware
    ///
    fn send(&mut self, command: &PyCommand) -> PyResult<()> {
        if let Some(port) = &mut self.port {
            let mut buf = [0u8; Command::MAX_SERIAL_SIZE];
            let msg = serialize::<Command>(&command.into(), &mut buf)
                .expect("serializing should just work");

            for i in 0..msg.len() {
                port.write(&msg[i..i + 1])?;
                thread::sleep(self.send_delay);
            }

            port.flush()?;

            Ok(())
        } else {
            Err(PyException::new_err("the socket was closed"))
        }
    }

    /// Receives information from the firmware.
    ///
    /// Raises errors on failed io operations and if it fails to deserialize
    /// a message.
    ///
    /// This method returns either a `Sensors` or a `Log`. Therefore,
    /// it is recommended to match its output a little like this:
    /// ```python
    /// >>> match firmware.recv():
    /// ...    case Sensors() as sensors:
    /// ...        ...
    /// ...    case Log() as log:
    /// ...        ...
    /// ```
    ///
    /// Returns:
    /// --------
    /// Union[Sensors, Log]
    ///     an event in the firmware
    ///
    fn recv(&mut self) -> PyResult<PyInfo> {
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
                Info::Sensors(sensors) => PyInfo::Sensors(sensors.into()),
                Info::Log(log) => PyInfo::Log(log.into()),
            })
        } else {
            Err(PyException::new_err("the socket was closed"))
        }
    }

    /// Closes the connection to the firmware.
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
///
/// It provides the means to send commands to and receive information from the
/// Xmaxx.
///
/// Usage:
/// ```python
/// >>> from xmaxx_python import *
/// >>>
/// >>> firmware = Firmware("/path/to/port")
/// >>>
/// >>> command = Command(42, 37, 37, 37, 37)
/// >>> firmware.send(command)
/// >>>
/// >>> match firmware.recv():
/// ...    case Sensors() as sensors:
/// ...        ...
/// ...    case Log() as Log:
/// ...        ...
/// ```
#[pymodule]
fn xmaxx_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyFirmware>()?;
    m.add_class::<PyCommand>()?;
    m.add_class::<PySensors>()?;
    m.add_class::<PyLog>()?;
    Ok(())
}
