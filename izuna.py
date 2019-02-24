
from ctypes import *
import math
import pythoncom
import pyHook
import threading
import time
import win32api
import win32con
import win32gui


class MouseEmulator:
    """Emulates mouse actions."""
    def __init__(self):
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

    def get_pointer_pos(self):
        class POINT(Structure):
            _fields_ = [("x", c_ulong),("y", c_ulong)]
        po = POINT()
        windll.user32.GetCursorPos(byref(po))
        return int(po.x), int(po.y)

    def move_pointer_pos(self, x, y, relative=False):
        # TODO: float available?
        # TODO: relative param not implemented
        windll.user32.SetCursorPos(int(x), int(y))
        return
    pass


class KeyStateMonitor:
    """Instantaneously monitors all states, if required."""
    def __init__(self, action_callback=None):
        self.rules = [
            # Key Name, Triggered Action, Virtual Key ID
            ('7', 'move-upper-left', (36, 0)),
            ('4', 'move-left', (37, 0)),
            ('1', 'move-lower-left', (35, 0)),
            ('2', 'move-down', (40, 0)),
            ('3', 'move-lower-right', (34, 0)),
            ('6', 'move-right', (39, 0)),
            ('9', 'move-upper-right', (33, 0)),
            ('8', 'move-up', (38, 0)),
            ('5', 'left-click', (12, 0)),
            ('\n', 'left-click', (13, 1)),
            ('0', 'left-click', (45, 0)),
            ('+', 'right-click', (106, 0)),
            ('/', 'middle-click', (111, 1)),
            ('*', 'wheel-scroll-up', (106, 0)),
            ('-', 'wheel-scroll-down', (109, 0)),
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
        if action_callback is None:
            action_callback = (lambda action, state: False)
        self.action_callback = action_callback
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
        if not self.is_monitoring():
            return True
        key = self.index_id_to_name[(k_id, k_ext)]
        action = self.index_id_to_action[(k_id, k_ext)]
        do_callback = self.action_states[action] != state
        self.key_states[key] = state
        self.action_states[action] = state
        if do_callback:
            self.action_callback(action, state)
        return False
    pass


key_mon = {
    '1': False,
    '2': False,
    '3': False,
    '4': False,
    '5': False,
    '6': False,
    '7': False,
    '8': False,
    '9': False,
    '0': False,
    '\n': False,
    '+': False,
    '/': False,
    '*': False,
    '-': False,
}

izuna_enabled = False

def judge_is_ctrl_key(key_id, ext):
    mmp = {
        (45, 0): '0',
        (35, 0): '1',
        (40, 0): '2',
        (34, 0): '3',
        (37, 0): '4',
        (12, 0): '5',
        (39, 0): '6',
        (36, 0): '7',
        (38, 0): '8',
        (33, 0): '9',
        (13, 1): '\n',
        (107, 0): '+',
        (111, 1): '/',
        (106, 0): '*',
        (109, 0): '-',
    }
    return mmp.get((key_id, ext), '')

class POINT(Structure):
    _fields_ = [("x", c_ulong),("y", c_ulong)]

def get_mouse_pos():
    po = POINT()
    windll.user32.GetCursorPos(byref(po))
    return int(po.x), int(po.y)

def mouse_move(x, y):
    windll.user32.SetCursorPos(int(x), int(y))
    return

def mouse_ldown():
    win32api.mouse_event(win32con.MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0)
    return

def mouse_lup():
    win32api.mouse_event(win32con.MOUSEEVENTF_LEFTUP, 0, 0, 0, 0)
    return

def num_lock_on():
    vk = win32api.GetKeyState(win32con.VK_NUMLOCK)
    return vk == 1 or vk == -127

class MouseClickController:
    def __init__(self):
        self.trig_stat = 0
        self.keys_trig = {'0': False, '\n': False}
        self.pressed = False
        self.lock = threading.Lock()
        return
    def trigger(self, key):
        if self.keys_trig[key]:
            return
        self.keys_trig[key] = True
        self.lock.acquire()
        if self.trig_stat == 0 and not self.pressed:
            mouse_ldown()
        self.trig_stat += 1
        self.lock.release()
        return
    def untrigger(self, key):
        if not self.keys_trig[key]:
            return
        self.keys_trig[key] = False
        self.lock.acquire()
        if self.trig_stat == 1 and not self.pressed:
            mouse_lup()
        self.trig_stat -= 1
        self.lock.release()
        return
    def press(self):
        self.lock.acquire()
        if self.trig_stat == 0 and not self.pressed:
            mouse_ldown()
        self.pressed = True
        self.lock.release()
        return
    def unpress(self):
        self.lock.acquire()
        if self.trig_stat == 0 and self.pressed:
            mouse_lup()
        self.pressed = False
        self.lock.release()
        return
    pass
mouse_clickcon = MouseClickController()

def onKeydownEvent(event):
    k_str = event.Key
    k_id = event.KeyID
    k_ext = event.Extended
    k_int = judge_is_ctrl_key(k_id, k_ext)
    if not izuna_enabled:
        return True
    if k_int != '':
        key_mon[k_int] = True
        if k_int in {'0', '\n'}:
            mouse_clickcon.trigger(k_int)
        elif k_int == '+':
                win32api.mouse_event(win32con.MOUSEEVENTF_RIGHTDOWN, 0, 0, 0, 0)
        elif k_int == '/':
                win32api.mouse_event(win32con.MOUSEEVENTF_MIDDLEDOWN, 0, 0, 0, 0)
        return False
    return True

def onKeyupEvent(event):
    k_str = event.Key
    k_id = event.KeyID
    k_ext = event.Extended
    k_int = judge_is_ctrl_key(k_id, k_ext)
    if not izuna_enabled:
        return True
    if k_int != '':
        key_mon[k_int] = False
        if k_int in {'0', '\n'}:
            mouse_clickcon.untrigger(k_int)
        elif k_int == '+':
            win32api.mouse_event(win32con.MOUSEEVENTF_RIGHTUP, 0, 0, 0, 0)
        elif k_int == '/':
            win32api.mouse_event(win32con.MOUSEEVENTF_MIDDLEUP, 0, 0, 0, 0)
        return False
    return True

def limit(a, x, y):
    return min(y, max(a, x))

def limit_abs(a, b):
    if b < 0:
        b = -b
    return limit(a, -b, b)

def sgn(a):
    if a > 0:
        return 1
    elif a < 0:
        return -1
    return 0

class Vector:
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
    def limit_abs(a, b):
        return Vector(limit_abs(a.x, b), limit_abs(a.y, b))
    pass

def get_mouse_pos_vector():
    vec = get_mouse_pos()
    vec = Vector(vec[0], vec[1])
    return vec

def mouse_con_d():
    global izuna_enabled
    # constants
    key_speed_vec = {
        '1': Vector(-2.6,  2.6),
        '2': Vector( 0.0,  2.6),
        '3': Vector( 2.6,  2.6),
        '4': Vector(-2.6,  0.0),
        '5': Vector( 0.0,  0.0),
        '6': Vector( 2.6,  0.0),
        '7': Vector(-2.6, -2.6),
        '8': Vector( 0.0, -2.6),
        '9': Vector( 2.6, -2.6),
    }
    deaccel_r = 9.2
    deaccel_f_r = 15.0
    max_speed = 3.5
    t_delay = 0.005
    pixel_rate = 1000
    window_size_abs = Vector(win32api.GetSystemMetrics(0), win32api.GetSystemMetrics(1))
    window_size = window_size_abs / pixel_rate
    scroll_a = 50
    scroll_dec_lim = 100
    scroll_dec_r = 0.02
    scroll_dec_a = 550
    # variables
    speed = Vector(0.0, 0.0)
    position = get_mouse_pos_vector() / pixel_rate
    k_pressed = False
    k_p_state = True
    scroll_v = 0.0
    last_time = time.time() - t_delay
    # loop
    while True:
        cur_time = time.time()
        t_past = cur_time - last_time
        last_time = cur_time
        # if not welcomed, skip
        if num_lock_on():
            izuna_enabled = False
            time.sleep(t_delay)
            continue
        izuna_enabled = True
        # define moving vector
        move_vec = Vector(0.0, 0.0)
        for i in range(1, 10):
            if key_mon[str(i)]:
                move_vec += key_speed_vec[str(i)]
        move_vec = move_vec.limit_abs(1.8)
        # adding to speed
        speed += move_vec * t_past
        v_sgn = Vector(sgn(speed.x), sgn(speed.y))
        # define de-acceleration rate
        deacc = - Vector(
            sgn(speed.x) if move_vec.x == 0.0 else 0.0,
            sgn(speed.y) if move_vec.y == 0.0 else 0.0
        ) * (deaccel_f_r if key_mon['5'] else deaccel_r)
        if sgn(speed.x) == - sgn(move_vec.x):
            deacc.x = - sgn(speed.x) * deaccel_f_r
        if sgn(speed.y) == - sgn(move_vec.y):
            deacc.y = - sgn(speed.y) * deaccel_f_r
        # appending deacceleration
        speed += deacc * t_past
        if sgn(speed.x) != v_sgn.x:
            speed.x = 0.0
        if sgn(speed.y) != v_sgn.y:
            speed.y = 0.0
        speed = speed.limit_abs(max_speed)
        # add speed to position
        last_pos = position
        position = get_mouse_pos_vector() / pixel_rate
        # if (last_pos - position).length() * pixel_rate <= 1.6:
        #     position = last_pos
        position += speed * t_past
        position.x = limit(position.x, 0, window_size.x)
        position.y = limit(position.y, 0, window_size.y)
        # setting mouse location
        # print('a: %s\tv: %s\tx: %s' % (str(move_vec + deacc), str(speed), str(position)))
        real_x = int(position.x * pixel_rate)
        real_y = int(position.y * pixel_rate)
        if last_pos != position:
            mouse_move(real_x, real_y)
        # press shortcut key
        do_press = False
        if key_mon['5']:
            if last_pos == position:
                if not k_pressed:
                    do_press = True
                    k_pressed = True
                    k_p_state = True
                else:
                    if k_p_state == True:
                        do_press = True
                        k_p_state = True
            else:
                if k_pressed and k_p_state:
                    do_press = True
                else:
                    k_pressed = True
                    k_p_state = False
        else:
            k_pressed = False
            k_p_state = last_pos != position
        # really press it
        if do_press:
            mouse_clickcon.press()
        else:
            mouse_clickcon.unpress()
        # manage scrolling
        scroll_p = False
        s_a = 0.0
        if key_mon['*']:
            s_a += scroll_a
            scroll_p = True
        if key_mon['-']:
            s_a -= scroll_a
            scroll_p = True
        scroll_v += s_a * t_past
        if not scroll_p or sgn(scroll_v) == - sgn(s_a):
            ss = sgn(scroll_v)
            scroll_v = abs(scroll_v)
            if scroll_v > scroll_dec_lim:
                scroll_v *= scroll_dec_r ** t_past
            else:
                scroll_v -= scroll_dec_a * t_past
            if scroll_v < 0:
                scroll_v = 0
            scroll_v *= ss
        win32api.mouse_event(win32con.MOUSEEVENTF_WHEEL, 0, 0, int(scroll_v))
        # sleep for a time break
        time.sleep(t_delay)
    return

print ('izuna Pointing Device Driver / 0.17')
print ('========================================')
print ('Author: jeffswt')
print ('Usage:  See README.md')

hm = pyHook.HookManager()
hm.KeyDown = onKeydownEvent
hm.KeyUp = onKeyupEvent
hm.HookKeyboard()
threading.Thread(target=mouse_con_d, args=[]).start()
pythoncom.PumpMessages()
