"""Simple gamepad/joystick test example."""

from __future__ import print_function
import inputs

EVENT_ABB = (
    # D-PAD, aka HAT
    ('Absolute-ABS_HAT0X', 'D-LR'),
    ('Absolute-ABS_HAT0Y', 'D-UD'),

    # Face Buttons
    ('Key-BTN_NORTH', 'Y'),
    ('Key-BTN_EAST', 'B'),
    ('Key-BTN_SOUTH', 'A'),
    ('Key-BTN_WEST', 'X'),

    # Other buttons
    ('Key-BTN_THUMBL', 'L3'),
    ('Key-BTN_THUMBR', 'R3'),
    ('Key-BTN_TL', 'LB'),
    ('Key-BTN_TR', 'RB'),
    ('Key-BTN_SELECT', 'Start'),
    ('Key-BTN_START', 'Back'),

    # Analog stick axes
    ('Absolute-ABS_X', 'L-LR'),
    ('Absolute-ABS_Y', 'L-UD'),
    ('Absolute-ABS_RX', 'R-LR'),
    ('Absolute-ABS_RY', 'R-UD'),
    ('Absolute-ABS_Z', 'LT'),
    ('Absolute-ABS_RZ', 'RT')
)

# This is to reduce noise from the PlayStation controllers
# For the Xbox controller, you can set this to 0
MIN_ABS_DIFFERENCE = 5

class JSTest(object):
    """Simple joystick test class."""
    def __init__(self, gamepad=None, abbrevs=EVENT_ABB):
        self.btn_state = {}
        self.old_btn_state = {}
        self.abs_state = {}
        self.old_abs_state = {}
        self.abbrevs = dict(abbrevs)
        for key, value in self.abbrevs.items():
            if key.startswith('Absolute'):
                self.abs_state[value] = 0
                self.old_abs_state[value] = 0
            if key.startswith('Key'):
                self.btn_state[value] = 0
                self.old_btn_state[value] = 0
        self._other = 0
        self.gamepad = gamepad
        if not gamepad:
            self._get_gamepad()

    def _get_gamepad(self):
        """Get a gamepad object."""
        try:
            self.gamepad = inputs.devices.gamepads[0]
        except IndexError:
            raise inputs.UnpluggedError("No gamepad found.")

    def handle_unknown_event(self, event, key):
        """Deal with unknown events."""
        if event.ev_type == 'Key':
            new_abbv = key #'Button_' + str(self._other)
            self.btn_state[new_abbv] = 0
            self.old_btn_state[new_abbv] = 0
        elif event.ev_type == 'Absolute':
            new_abbv = 'Axis_' + str(self._other)
            self.abs_state[new_abbv] = 0
            self.old_abs_state[new_abbv] = 0
        else:
            return None

        self.abbrevs[key] = new_abbv
        self._other += 1

        return self.abbrevs[key]

    def process_event(self, event):
        """Process the event into a state."""
        if event.ev_type == 'Sync':
            return
        if event.ev_type == 'Misc':
            return
        key = event.ev_type + '-' + event.code
        try:
            abbv = self.abbrevs[key]
        except KeyError:
            abbv = self.handle_unknown_event(event, key)
            if not abbv:
                return
        if event.ev_type == 'Key':
            self.old_btn_state[abbv] = self.btn_state[abbv]
            self.btn_state[abbv] = event.state
        if event.ev_type == 'Absolute':
            self.old_abs_state[abbv] = self.abs_state[abbv]
            self.abs_state[abbv] = event.state
        return self.output_state(event.ev_type, abbv)

    def format_state(self):
        """Format the state."""
        output_dict = {**self.abs_state, **self.btn_state}
        return output_dict

    def output_state(self, ev_type, abbv):
        """Print out the output state."""
        if ev_type == 'Key':
            if self.btn_state[abbv] != self.old_btn_state[abbv]:
                return self.format_state()

        if abbv[0] == 'D':
            return self.format_state()

        difference = self.abs_state[abbv] - self.old_abs_state[abbv]
        if abs(difference) > MIN_ABS_DIFFERENCE:
            return self.format_state()

    def process_events(self):
        """Process available events."""
        try:
            events = self.gamepad.read()
        except EOFError:
            events = []
        for event in events:
            result = self.process_event(event)
            if result:
                return result

def main():
    """Process all events forever."""
    jstest = JSTest()
    
    A = 0
    UD = 0
    LR = 0
    
    while True:
        event_dict = jstest.process_events()
        if event_dict:
            A = event_dict['A']
            UD = event_dict['R-UD']
            LR = event_dict['L-LR']
            print(A)
            print(UD)
            print(LR)

if __name__ == "__main__":
    main()
