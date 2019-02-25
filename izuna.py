
from ctypes import *
import math
import pythoncom
import pyHook
import threading
import time
import win32api
import win32con
import win32gui


class Vector:
    """Vector implementation."""
    def __init__(a, x=0, y=0):
        if type(x) not in {int, float} or type(y) not in {int, float}:
            raise TypeError('requires real number as arguments')
        a.x = float(x)
        a.y = float(y)
        return

    def __str__(a):
        return '(%.4f, %.4f)' % (a.x, a.y)

    def __repr__(a):
        return 'Vector(%s, %s)' % (str(a.x), str(a.y))

    def __eq__(a, b):
        return a.x == b.x and a.y == b.y

    def __bool__(a):
        return a.x != 0 or a.y != 0

    def __neg__(a):
        return Vector(-a.x, -a.y)

    def __add__(a, b):
        return Vector(a.x + b.x, a.y + b.y)

    def __sub__(a, b):
        return Vector(a.x - b.x, a.y - b.y)

    def __mul__(a, b):
        if type(a) == type(b):
            return float(a.x * b.x + a.y * b.y)
        elif type(b) not in {int, float}:
            raise TypeError('requires real number or vector as arguments')
        return Vector(a.x * b, a.y * b)

    def __truediv__(a, b):
        if type(b) not in {int, float}:
            raise TypeError('requires real number as arguments')
        return Vector(a.x / b, a.y / b)

    def __div__(a, b):
        return a.__truediv__(b)

    def __rmul__(a, b):
        return a * b

    def length(a):
        return math.sqrt(a.x * a.x + a.y * a.y)
    pass


class MouseEmulator:
    """Emulates mouse actions."""
    def __init__(self):
        self.x, self.y = self.get_pointer_pos()
        self.pos_buffer = (0.0, 0.0)  # Move buffer
        self.wheel_buffer = 0.0  # Wheel buffer
        self.frame_rate = 432  # Frames per second
        self.frame_time = 1.0 / self.frame_rate  # Time per frame
        return

    def change_key_state(self, key, state):
        mapping = {
            ('left', True): win32con.MOUSEEVENTF_LEFTDOWN,
            ('left', False): win32con.MOUSEEVENTF_LEFTUP,
            ('right', True): win32con.MOUSEEVENTF_RIGHTDOWN,
            ('right', False): win32con.MOUSEEVENTF_RIGHTUP,
            ('middle', True): win32con.MOUSEEVENTF_MIDDLEDOWN,
            ('middle', False): win32con.MOUSEEVENTF_MIDDLEUP,
        }
        if (key, state) not in mapping:
            return False
        win32api.mouse_event(mapping[(key, state)], 0, 0, 0, 0)
        return True

    def get_pointer_pos(self, x=0, y=0):
        class POINT(Structure):
            _fields_ = [("x", c_ulong), ("y", c_ulong)]
        po = POINT()
        windll.user32.GetCursorPos(byref(po))
        nx, ny = int(po.x), int(po.y)
        if nx != math.floor(x) and nx != math.ceil(x):
            x = nx
        if ny != math.floor(y) and ny != math.floor(y):
            y = ny
        return x, y

    def move_pointer_pos(self, x, y, relative=False):
        _x, _y = self.get_pointer_pos(self.x, self.y)  # Previous state
        if relative:
            x, y = _x + x, _y - y
            self.x, self.y = x, y
        if (int(x), int(y)) != (int(_x), int(_y)):
            windll.user32.SetCursorPos(int(x), int(y))
        return

    def move_pointer_pos_async(self, x, y):
        """Move pointer position, asynchronously, forced relative position."""
        self.pos_buffer = (self.pos_buffer[0] + x, self.pos_buffer[1] + y)
        return

    def scroll_wheel(self, distance):
        win32api.mouse_event(win32con.MOUSEEVENTF_WHEEL, 0, 0, int(distance))
        return

    def scroll_wheel_async(self, distance):
        self.wheel_buffer += distance
        return

    def emulator_worker(self):
        while True:
            x, y = self.pos_buffer
            self.pos_buffer = (0.0, 0.0)
            self.move_pointer_pos(x, y, relative=True)
            w = self.wheel_buffer
            self.wheel_buffer = 0.0
            self.scroll_wheel(w)
            time.sleep(self.frame_time)
        return
    pass


class KeyStateMonitor:
    """Instantaneously monitors all states, if required."""
    def __init__(self, action_handler=None):
        self.rules = [
            # Key Name, Triggered Action, Virtual Key ID
            # Prepend '$' on key name to disable blocking
            ('num-7', 'move-upper-left', (36, 0)),
            ('num-4', 'move-left', (37, 0)),
            ('num-1', 'move-lower-left', (35, 0)),
            ('num-2', 'move-down', (40, 0)),
            ('num-3', 'move-lower-right', (34, 0)),
            ('num-6', 'move-right', (39, 0)),
            ('num-9', 'move-upper-right', (33, 0)),
            ('num-8', 'move-up', (38, 0)),
            ('num-5', 'left-click', (12, 0)),
            ('num-enter', 'left-click', (13, 1)),
            ('num-0', 'left-click', (45, 0)),
            ('num-del', 'left-click', (46, 0)),
            ('num-plus', 'right-click', (107, 0)),
            ('num-slash', 'middle-click', (111, 1)),
            ('num-asterisk', 'wheel-scroll-up', (106, 0)),
            ('num-hyphen', 'wheel-scroll-down', (109, 0)),
            ('$left-alt', 'speed-switch', (164, 0))
        ]
        # Generate indices
        self.index_ids = set(i[2] for i in self.rules)
        self.index_id_to_name = dict((i[2], i[0]) for i in self.rules)
        self.index_name_to_action = dict((i[0], i[1]) for i in self.rules)
        self.index_id_to_action = dict((i[2], i[1]) for i in self.rules)
        # Key states
        self.key_states = dict((i[2], False) for i in self.rules)
        self.action_states = dict((i[1], False) for i in self.rules)
        # Callback
        self.action_callback = action_handler
        return

    def is_monitoring(self):
        """Checks if izuna is still on (num lock is off)."""
        vk = win32api.GetKeyState(win32con.VK_NUMLOCK)
        return vk == 1 or vk == -127

    def on_key_event(self, event, state):
        """Key up/down event handler, returns False if key captured."""
        k_str, k_id, k_ext = event.Key, event.KeyID, event.Extended
        if (k_id, k_ext) not in self.index_ids:
            return True
        if self.is_monitoring():
            return True
        key = self.index_id_to_name[(k_id, k_ext)]
        action = self.index_id_to_action[(k_id, k_ext)]
        do_callback = self.action_states[action] != state
        self.key_states[key] = state
        self.action_states[action] = state
        if do_callback:
            if self.action_callback is not None:
                self.action_callback.callback(action, state)
        # Keys with '$' prefix would not be blocked
        return key[0] == '$'

    def load_hook(self):
        hm = pyHook.HookManager()

        def on_keydown_event(event):
            return self.on_key_event(event, True)

        def on_keyup_event(event):
            return self.on_key_event(event, False)

        hm.KeyDown = on_keydown_event
        hm.KeyUp = on_keyup_event
        hm.HookKeyboard()
        # pythoncom.PumpMessages()
        return
    pass


class ActionHandler:
    """Handles actions initiated by key presses."""
    def __init__(self, emulator=None):
        # Constants
        self.move_speed = {
            'move-upper-left': Vector(-1.5, 1.3),
            'move-left': Vector(-2.1, 0),
            'move-lower-left': Vector(-1.5, -1.2),
            'move-down': Vector(0, -1.9),
            'move-lower-right': Vector(1.5, -1.2),
            'move-right': Vector(2.1, 0),
            'move-upper-right': Vector(1.5, 1.3),
            'move-up': Vector(0, 2.0),
        }
        self.scroll_speed = {
            'wheel-scroll-up': 2.0,
            'wheel-scroll-down': -2.0,
        }
        self.keys = {
            'left-click': 'left',
            'right-click': 'right',
            'middle-click': 'middle',
        }
        # Acceleration functions
        self.func_accel = (lambda _: math.log(_ + 1, 1.08) ** 0.56 * 0.358)
        self.func_accel_r = (lambda _: 1.08 ** ((_ / 0.362) ** (1 / 0.56)) - 1)
        self.func_decel = (lambda l, _: max(0.0, l - 7.9 * _))
        self.vec_scale = 404.200
        self.vec_switch_scale = 0.297
        self.func_waccel = (lambda _: math.sqrt(_ * 1.8) * 1.3)
        self.func_waccel_r = (lambda _: (_ / 1.3) ** 2 / 1.8)
        self.func_wdecel = (lambda l, _: max(0.0, l - 5.43 * _))
        self.wheel_scale = 1579.3
        self.wheel_switch_scale = 2.11
        # Lists
        #   enabled: current status, True if pressed down
        #   time: when did the current status started
        #   combo: current combo
        #   combo_last: last combo achieved before state change
        self.vec_enabled = dict((i, False) for i in self.move_speed)
        self.vec_time = dict((i, 0.0) for i in self.move_speed)
        self.vec_combo = dict((i, 0.0) for i in self.move_speed)
        self.vec_combo_last = dict((i, 0.0) for i in self.move_speed)
        self.wheel_enabled = dict((i, False) for i in self.scroll_speed)
        self.wheel_time = dict((i, 0.0) for i in self.scroll_speed)
        self.wheel_combo = dict((i, 0.0) for i in self.scroll_speed)
        self.wheel_combo_last = dict((i, 0.0) for i in self.scroll_speed)
        # Mouse emulator
        self.emulator = emulator
        # Time related
        self.last_frame_time = 0.0
        self.speed_switch = False
        return

    def callback(self, action, state):
        if action in self.move_speed:
            if self.vec_enabled[action] != state:
                self.vec_combo_last[action] = self.vec_combo[action]
            self.vec_enabled[action] = state
            self.vec_time[action] = time.time()
        elif action in self.scroll_speed:
            if self.wheel_enabled[action] != state:
                self.wheel_combo_last[action] = self.wheel_combo[action]
            self.wheel_enabled[action] = state
            self.wheel_time[action] = time.time()
        elif action in self.keys:
            self.emulator.change_key_state(self.keys[action], state)
        elif action == 'speed-switch':
            self.speed_switch = state
        return

    def render_frame(self):
        cur_time = time.time()
        frame_time = (self.emulator.frame_time if self.last_frame_time == 0.0
                      else cur_time - self.last_frame_time)
        self.last_frame_time = cur_time
        # Process cursor position
        vec = Vector(0.0, 0.0)
        for action in self.move_speed:
            delta_tm = cur_time - self.vec_time[action]
            combo = 0.0
            if self.vec_enabled[action]:
                last_tm = self.func_accel_r(self.vec_combo_last[action])
                combo = self.func_accel(last_tm + delta_tm)
            else:
                combo = self.func_decel(self.vec_combo_last[action], delta_tm)
            vec = vec + self.move_speed[action] * combo
            self.vec_combo[action] = combo
        vec *= self.vec_scale
        if self.speed_switch:
            vec *= self.vec_switch_scale
        vec *= frame_time
        self.emulator.move_pointer_pos_async(vec.x, vec.y)
        # Process wheel status
        wheel = 0.0
        for action in self.scroll_speed:
            delta_tm = cur_time - self.wheel_time[action]
            combo = 0.0
            if self.wheel_enabled[action]:
                last_tm = self.func_waccel_r(self.wheel_combo_last[action])
                combo = self.func_waccel(last_tm + delta_tm)
            else:
                combo = self.func_wdecel(self.wheel_combo_last[action],
                                         delta_tm)
            wheel = wheel + self.scroll_speed[action] * combo
            self.wheel_combo[action] = combo
        wheel *= self.wheel_scale
        if self.speed_switch:
            wheel *= self.wheel_switch_scale
        wheel *= self.emulator.frame_time
        self.emulator.scroll_wheel_async(wheel)
        return
    pass


def main():
    print('izuna Pointing Device Driver / 1.02')
    print('========================================')
    print('Author: jeffswt')
    print('Usage:  See README.md')
    mouse_emulator = MouseEmulator()
    action_handler = ActionHandler(emulator=mouse_emulator)
    key_state_monitor = KeyStateMonitor(action_handler=action_handler)
    key_state_monitor.load_hook()

    def render_frame(action_handler, mouse_emulator):
        while True:
            action_handler.render_frame()
            time.sleep(mouse_emulator.frame_time)
        return

    render_thread = threading.Thread(
        target=render_frame, args=[action_handler, mouse_emulator])
    mouse_pos_thread = threading.Thread(
        target=mouse_emulator.emulator_worker, args=[])
    render_thread.start()
    mouse_pos_thread.start()
    pythoncom.PumpMessages()
    render_thread.join()
    mouse_pos_thread.join()
    return


if __name__ == "__main__":
    main()
