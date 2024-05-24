# Joystick/Gamepad Test Example

This is a simple example of testing joystick/gamepad inputs using Python. It asumes a logitech controller.

## Usage

1. **Requirements:**
    - Python 3.x
    - `inputs` library (Install using `pip install inputs`)

2. **Running the script:**
    ```bash
    python joy.py
    ```

## Description

This script reads inputs from a connected joystick or gamepad and prints the state of the buttons and axes.

## Events and Abbreviations (keys)

- **Main Buttons:**
    - `Y`: Button Y
    - `B`: Button B
    - `A`: Button A
    - `X`: Button X
    - `LB`: Left bumper
    - `RB`: Right bumper

- **Analog Stick Axes:**
    - `L-LR`: Left analog stick left-right axis     (-32768 - 32767)
    - `L-UD`: Left analog stick up-down axis        (-32768 - 32767)
    - `R-LR`: Right analog stick left-right axis    (-32768 - 32767)
    - `R-UD`: Right analog stick up-down axis       (-32768 - 32767)
    - `LT`: Left trigger                            (0 - 255)
    - `RT`: Right trigger                           (0 - 255)

- **D-PAD:**
    - `D-LR`: Left-right directional input
    - `D-UD`: Up-down directional input

- **Other Buttons:**
    - `L3`: Left analog stick button
    - `R3`: Right analog stick button
    - `Start`: Start button
    - `Back`: Back/Select button


## Customization

- `MIN_ABS_DIFFERENCE`: This variable is used to reduce noise.

## Class: `JSTest`

### Methods:

- `__init__(self, gamepad=None, abbrevs=EVENT_ABB)`: Initialize the joystick test object.
- `_get_gamepad(self)`: Get the gamepad object.
- `handle_unknown_event(self, event, key)`: Deal with unknown events.
- `process_event(self, event)`: Process the event into a state.
- `format_state(self)`: Format the state.
- `output_state(self, ev_type, abbv)`: Print out the output state.
- `process_events(self)`: Process available events.

## Example

```python
if __name__ == "__main__":
    jstest = JSTest() # create joy object
    
    A = 0
    UD = 0
    LR = 0
    
    while True:
        event_dict = jstest.process_events() # read buttons
        if event_dict:
            A = event_dict['A']
            UD = event_dict['R-UD']
            LR = event_dict['L-LR']
            print(A)
            print(UD)
            print(LR)
