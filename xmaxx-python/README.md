# xmaxx_python

Python bindings to talk with the Xmaxx's firmware.

## Dependencies

Rust must be [installed](https://www.rust-lang.org/tools/install).

## Build and install

1. `cd` into this directory.

2. Create a new vitural environment and acitvate it.
   ```shell
   python3 -m venv venv && . venv/bin/activate
   ```

3. Build and install using `pip`.
   ```shell
   python3 -m pip install .
   ```

## Usage

For usage in Python, see the module's documentation.

## Documentation

The documentation can be seen from Python with `help()` or `python3 -m pydoc`.
To build the Rust version, run `cargo doc --document-private-items`.
