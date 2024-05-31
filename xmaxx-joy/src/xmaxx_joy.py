import pygame
from xmaxx_python import Command


class XmaxxJoy:
    """A joystick to control the Xmaxx remotely."""

    # Logitech switch: x mode: off
    A_BTN = 0
    B_BTN = 1
    X_BTN = 2
    Y_BTN = 3
    LB_BTN = 4
    RB_BTN = 5
    L_BTN = 9
    R_BTN = 10
    START = 7
    BACK = 6
    CENTRAL = 8

    L_LR_AX = 0
    L_UD_AX = 1
    R_LR_AX = 3
    R_UD_AX = 4
    LT_AX = 2
    RT_AX = 5

    DPAD_HAT = 0

    def __init__(self, firmware):
        """Initializes the joystick.

        Parameters:
        -----------
        firmware
            the firmware with which to communicate
        """
        pygame.init()
        self._joy = pygame.joystick.Joystick(0)
        self._firmware = firmware

        self._should_listen_and_command = False

    def _deadman_pressed(self):
        """Checks if the deadman switch is pressed."""
        return self._joy.get_button(self.X_BTN)

    def parse_command(self):
        """Reads the joystick and returns a command."""
        forward = self._joy.get_axis(self.RT_AX)
        backward = self._joy.get_axis(self.LT_AX)
        net = forward - backward
        steering = self._joy.get_axis(self.L_LR_AX)
        return Command(steering, net, net, net, net)

    def listen_and_command(self, freq=30):
        """Listens to the joystick and sends commands to the firmware.

        Parameters:
        -----------
        freq: int = 30
            the polling frequency of the loop

        This method contains an infinite loop and will monopilize the calling
        thread until `.stop_listening()` is called.
        """
        clock = pygame.time.Clock()
        self._should_listen_and_command = True

        while self._should_listen_and_command:
            if self._deadman_pressed():
                command = self.parse_command()
                try:
                    self._firmware.send(command)
                except Exception as _:
                    pass

            pygame.event.pump()
            clock.tick(freq)

    def stop_listening(self):
        """Interrupts the listen and command loop."""
        self._should_listen_and_command = False


if __name__ == "__main__":
    class DummyFirmware:
        def send(self, command):
            print("Sent", command)

    joy = XmaxxJoy(DummyFirmware())
    joy.listen_and_command()
